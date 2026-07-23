#!/usr/bin/env python3
"""
analyze_repo.py — SkillForge Analysis stage.

Shallow-clones a GitHub repo (or scans a local path), inspects it for
agent-facing surfaces, classifies it, and emits a single JSON object on stdout
describing how to turn it into an agent skill.

Contract (stdout is ONE json object, nothing else):
{
  "ok": true,
  "name": "supermemory",
  "description": "...",
  "classification": "ApiOrMcp" | "CliOnly" | "Unknown",
  "confidence": 0.0-1.0,
  "base_url_hint": "http://localhost:PORT" | null,
  "auth_hint": {"header": "...", "env": "..."} | null,
  "candidate_routes": {"route_name": "METHOD /path"},
  "surfaces": {"openapi": bool, "mcp": bool, "dockerfile": bool,
               "rest_hints": bool, "readme": bool},
  "missing_agent_surfaces": ["mcp_server", "openapi_spec", ...],
  "risk_signals": ["exploit-tooling", "writes-to-network", ...],
  "language": "python|rust|go|...",
  "notes": ["..."]
}

Any failure still yields a JSON object with "ok": false and "error".
Never raises to the caller — the Rust Steward relies on always getting JSON.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

CLONE_TIMEOUT = 120  # seconds
MAX_README_BYTES = 200_000

# --- helpers ---------------------------------------------------------------

def _emit(obj):
    sys.stdout.write(json.dumps(obj))
    sys.stdout.flush()
    sys.exit(0 if obj.get("ok") else 0)  # always exit 0; ok flag carries status


def _fail(msg, **extra):
    out = {"ok": False, "error": msg}
    out.update(extra)
    _emit(out)


def _slug_from_url(url):
    m = re.search(r"github\.com[/:]+([^/]+)/([^/.]+)", url)
    if m:
        return m.group(2).lower().replace("_", "-")
    return "unknown-skill"


def _shallow_clone(url, dest):
    return subprocess.run(
        ["git", "clone", "--depth", "1", "--quiet", url, dest],
        capture_output=True, text=True, timeout=CLONE_TIMEOUT,
    )


def _read_text(path, limit=MAX_README_BYTES):
    try:
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            return f.read(limit)
    except OSError:
        return ""


def _find(root, names):
    """Return first matching file (case-insensitive) among names, shallow-ish."""
    lowered = {n.lower() for n in names}
    for dirpath, dirs, files in os.walk(root):
        # keep the walk cheap: skip vendor/build dirs and don't go too deep
        depth = Path(dirpath).relative_to(root).parts
        if len(depth) > 3:
            dirs[:] = []
            continue
        dirs[:] = [d for d in dirs if d not in
                   {".git", "node_modules", "target", "dist", "build",
                    ".venv", "venv", "__pycache__", "vendor"}]
        for f in files:
            if f.lower() in lowered:
                return os.path.join(dirpath, f)
    return None


def _glob_any(root, patterns):
    hits = []
    for dirpath, dirs, files in os.walk(root):
        depth = Path(dirpath).relative_to(root).parts
        if len(depth) > 3:
            dirs[:] = []
            continue
        dirs[:] = [d for d in dirs if d not in
                   {".git", "node_modules", "target", "dist", "build",
                    ".venv", "venv", "__pycache__", "vendor"}]
        for f in files:
            for pat in patterns:
                if re.search(pat, f, re.IGNORECASE):
                    hits.append(os.path.join(dirpath, f))
    return hits


def _detect_language(root):
    markers = [
        ("Cargo.toml", "rust"), ("go.mod", "go"),
        ("package.json", "typescript"), ("pyproject.toml", "python"),
        ("requirements.txt", "python"), ("mix.exs", "elixir"),
        ("pom.xml", "java"), ("Gemfile", "ruby"),
    ]
    for fname, lang in markers:
        if _find(root, [fname]):
            return lang
    return "unknown"


# --- OpenAPI / MCP / route extraction --------------------------------------

def _extract_openapi_routes(path):
    routes = {}
    base = None
    try:
        text = _read_text(path)
        # cheap YAML/JSON path extraction without pulling deps
        data = None
        if path.endswith((".yaml", ".yml")):
            try:
                import yaml  # optional
                data = yaml.safe_load(text)
            except Exception:
                data = None
        else:
            try:
                data = json.loads(text)
            except Exception:
                data = None
        if isinstance(data, dict):
            paths = data.get("paths", {})
            servers = data.get("servers") or []
            if servers and isinstance(servers, list):
                base = servers[0].get("url") if isinstance(servers[0], dict) else None
            for p, methods in list(paths.items())[:40]:
                if not isinstance(methods, dict):
                    continue
                for method in methods:
                    if method.lower() in {"get", "post", "put", "patch", "delete"}:
                        name = _route_name(method, p)
                        routes[name] = f"{method.upper()} {p}"
        else:
            # regex fallback for pathish lines
            for m in re.finditer(r'"(/[\w/{}\-.]+)"\s*:\s*{', text):
                routes[_route_name("get", m.group(1))] = f"GET {m.group(1)}"
    except Exception:
        pass
    return routes, base


def _route_name(method, path):
    seg = [s for s in re.split(r"[/{}]", path) if s and not s.startswith("$")]
    tail = "_".join(seg[-2:]) if seg else "root"
    tail = re.sub(r"[^a-z0-9_]", "_", tail.lower())
    prefix = "" if method.lower() == "get" else method.lower() + "_"
    return (prefix + tail)[:48] or "route"


def _scan_rest_hints(root):
    """Grep for framework route declarations to synthesize candidate routes."""
    routes = {}
    patterns = [
        # FastAPI / Flask
        (r'@\w+\.(get|post|put|patch|delete)\(\s*["\']([^"\']+)["\']', True),
        # Express / Fastify
        (r'\b(?:app|router)\.(get|post|put|patch|delete)\(\s*["\']([^"\']+)["\']', True),
        # Axum/Actix (rust) route("/path", get(..))
        (r'\.route\(\s*["\']([^"\']+)["\']\s*,\s*(get|post|put|patch|delete)', False),
    ]
    code_files = _glob_any(root, [r"\.py$", r"\.js$", r"\.ts$", r"\.rs$", r"\.go$"])
    for fp in code_files[:120]:
        text = _read_text(fp, 60_000)
        for pat, method_first in patterns:
            for m in re.finditer(pat, text):
                if method_first:
                    method, path = m.group(1), m.group(2)
                else:
                    path, method = m.group(1), m.group(2)
                if not path.startswith("/"):
                    continue
                routes[_route_name(method, path)] = f"{method.upper()} {path}"
                if len(routes) >= 30:
                    return routes
    return routes


def _detect_port(root):
    for fp in _glob_any(root, [r"docker-compose", r"Dockerfile", r"\.env",
                               r"\.py$", r"\.js$", r"\.ts$", r"\.go$", r"\.rs$"])[:80]:
        text = _read_text(fp, 40_000)
        m = re.search(r"(?:PORT|port|listen|EXPOSE)\D{0,8}(\d{2,5})", text)
        if m:
            p = int(m.group(1))
            if 1024 <= p <= 65535:
                return p
    return None


RISK_KEYWORDS = {
    "exploit-tooling": r"\b(exploit|metasploit|payload|reverse[- ]?shell|msfvenom)\b",
    "sql-injection": r"\bsqlmap|sql injection\b",
    "credential-access": r"\b(password|secret|api[_-]?key|token|credential)s?\b",
    "network-mutation": r"\b(DELETE|drop table|rm -rf|shutdown|format)\b",
    "arbitrary-exec": r"\b(eval\(|exec\(|subprocess|os\.system|child_process)\b",
}


def _risk_signals(readme, root):
    signals = []
    corpus = readme[:50_000]
    for label, pat in RISK_KEYWORDS.items():
        if re.search(pat, corpus, re.IGNORECASE):
            signals.append(label)
    return signals


# --- nuclei file scan (native, no docker) ----------------------------------

NUCLEI_TEMPLATES = os.environ.get(
    "SKILLFORGE_NUCLEI_TEMPLATES", "/root/nuclei-templates/file/")


def _betterleaks_scan(root):
    """Docker-run betterleaks secret scan against the raw clone, before
    anything is ever built or executed. Best-effort; never raises. A
    critical/verified secret here is a hard-fail signal upstream."""
    if not shutil.which("docker"):
        return {"ran": False, "reason": "docker not installed"}
    try:
        res = subprocess.run(
            ["docker", "run", "--rm", "-v", f"{root}:/repo:ro",
             "ghcr.io/betterleaks/betterleaks:latest", "dir", "/repo"],
            capture_output=True, text=True, timeout=120,
        )
    except subprocess.TimeoutExpired:
        return {"ran": False, "reason": "timeout"}
    except Exception as e:
        return {"ran": False, "reason": f"{type(e).__name__}: {e}"}
    out = res.stdout + res.stderr
    verified = out.count("Verified: true") + out.count("verified: true")
    total = out.count("┌─")
    return {"ran": True, "total": total, "verified": verified}


def _malware_signature_scan(root):
    """Cheap, targeted static red flags beyond nuclei's generic templates:
    known malicious-package patterns, obfuscated payload droppers, reverse
    shells. Not a substitute for the full OKF chain (Strix/Metasploit run
    later against a *booted* instance) -- this is the pre-execution gate,
    so it only ever reads source text, never runs anything from the repo."""
    patterns = {
        "reverse-shell": r"(nc\s+-e|/bin/sh\s+-i|bash\s+-i\s+>&|socket\.socket\(.*SOCK_STREAM.*\).*connect\(.*\))",
        "obfuscated-eval": r"eval\(\s*(atob|base64|Buffer\.from)\(",
        "crypto-miner": r"(stratum\+tcp://|xmrig|minerd)",
        "self-modifying-install-hook": r"(postinstall|preinstall)\"\s*:\s*\".*(curl|wget|eval)",
        "credential-exfil": r"(webhook\.site|requestbin|ngrok\.io)/[a-zA-Z0-9]{6,}",
    }
    hits = []
    for dirpath, _dirs, files in os.walk(root):
        if "/.git" in dirpath:
            continue
        for fn in files:
            if not fn.endswith((".py", ".js", ".ts", ".sh", ".json", ".rs", ".go")):
                continue
            fp = os.path.join(dirpath, fn)
            try:
                text = Path(fp).read_text(errors="ignore")[:200_000]
            except Exception:
                continue
            for label, pat in patterns.items():
                if re.search(pat, text, re.IGNORECASE):
                    hits.append({"signal": label, "file": os.path.relpath(fp, root)})
                    if len(hits) >= 25:
                        return hits
    return hits


def _security_prescan(root):
    """Mandatory Stage 0.5: runs on the raw clone before anything is ever
    built or executed. Hard-fail (never soft-fallback) on any verified
    secret, malware signature, or nuclei-critical finding -- this is the one
    gate the pipeline cannot proceed past, since everything downstream
    (boot + discovery) means running the repo's own code."""
    betterleaks = _betterleaks_scan(root)
    nuclei = _nuclei_scan(root)
    malware_hits = _malware_signature_scan(root)
    reasons = []
    if betterleaks.get("verified", 0) > 0:
        reasons.append(f"betterleaks found {betterleaks['verified']} verified secret(s)")
    if nuclei.get("critical", 0) > 0:
        reasons.append(f"nuclei found {nuclei['critical']} critical finding(s)")
    if malware_hits:
        signals = sorted({h["signal"] for h in malware_hits})
        reasons.append(f"malware signature(s) detected: {', '.join(signals)}")
    return {
        "betterleaks": betterleaks,
        "nuclei": nuclei,
        "malware_signatures": malware_hits,
        "hard_fail": bool(reasons),
        "reasons": reasons,
    }


def _nuclei_scan(root):
    """Run nuclei's native file templates over the repo to find secrets, keys,
    and misconfigs. No docker, no HTTP target. Best-effort; never raises."""
    if not shutil.which("nuclei"):
        return {"ran": False, "reason": "nuclei not installed"}
    if not os.path.isdir(NUCLEI_TEMPLATES):
        return {"ran": False, "reason": "templates missing"}
    try:
        res = subprocess.run(
            ["nuclei", "-file", "-target", root, "-t", NUCLEI_TEMPLATES,
             "-jsonl", "-silent", "-no-color", "-duc"],
            capture_output=True, text=True, timeout=180,
        )
    except subprocess.TimeoutExpired:
        return {"ran": False, "reason": "timeout"}
    except Exception as e:
        return {"ran": False, "reason": f"{type(e).__name__}: {e}"}
    sev = {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0}
    findings = []
    for line in res.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            f = json.loads(line)
        except Exception:
            continue
        s = (f.get("info", {}) or {}).get("severity", "info").lower()
        if s in sev:
            sev[s] += 1
        if len(findings) < 25:
            findings.append({
                "template": f.get("template-id"),
                "severity": s,
                "matched": f.get("matched-at") or f.get("host"),
            })
    return {
        "ran": True,
        "total": sum(sev.values()),
        "by_severity": sev,
        "critical": sev["critical"],
        "high": sev["high"],
        "findings": findings,
    }


# --- main ------------------------------------------------------------------

def main():
    if len(sys.argv) < 2:
        _fail("usage: analyze_repo.py <github-url|local-path>")

    target = sys.argv[1].strip()
    tmp = None
    try:
        if os.path.isdir(target):
            root = target
        else:
            if not re.match(r"https?://|git@", target):
                _fail("not a URL or existing path", target=target)
            tmp = tempfile.mkdtemp(prefix="skillforge-")
            root = os.path.join(tmp, "repo")
            res = _shallow_clone(target, root)
            if res.returncode != 0:
                _fail("clone failed", detail=res.stderr.strip()[:500])

        name = _slug_from_url(target) if not os.path.isdir(target) else Path(root).name.lower()

        # Stage 0.5: mandatory pre-execution security scan. Runs before any
        # further processing, on the raw clone, before anything from this
        # repo is ever built or executed.
        security_prescan = _security_prescan(root)
        if security_prescan["hard_fail"]:
            if tmp and os.path.isdir(tmp):
                shutil.rmtree(tmp, ignore_errors=True)
            _emit({
                "ok": False,
                "error": "security prescan hard-failed: " + "; ".join(security_prescan["reasons"]),
                "security_prescan": security_prescan,
            })

        readme_path = _find(root, ["README.md", "README.rst", "README.txt", "README"])
        readme = _read_text(readme_path) if readme_path else ""
        description = ""
        if readme:
            for line in readme.splitlines():
                s = line.strip().lstrip("#").strip()
                if (s and len(s) > 12 and not s.startswith(("!", "<", "[", "|", "-", "="))
                        and "://" not in s.split(" ")[0]):
                    description = s[:280]
                    break

        # surfaces
        openapi_path = _find(root, ["openapi.json", "openapi.yaml", "openapi.yml",
                                    "swagger.json", "swagger.yaml"])
        mcp_path = _find(root, ["mcp.json", "mcp_server.py", "server.json"]) \
            or bool(re.search(r"\bmodelcontextprotocol|@modelcontextprotocol|mcp\.server\b",
                              readme, re.IGNORECASE)) and "readme-mcp"
        dockerfile = _find(root, ["Dockerfile", "docker-compose.yml", "docker-compose.yaml"])

        routes = {}
        base_url_hint = None
        if openapi_path:
            routes, base_url_hint = _extract_openapi_routes(openapi_path)
        rest_routes = _scan_rest_hints(root)
        for k, v in rest_routes.items():
            routes.setdefault(k, v)

        has_mcp = bool(mcp_path)
        has_openapi = bool(openapi_path)
        has_rest = bool(rest_routes)

        port = _detect_port(root)
        if not base_url_hint:
            base_url_hint = f"http://localhost:{port}" if port else None

        # classification + confidence
        if has_openapi or has_mcp or (has_rest and base_url_hint):
            classification = "ApiOrMcp"
            confidence = 0.55
            if has_openapi:
                confidence += 0.25
            if has_mcp:
                confidence += 0.15
            if base_url_hint:
                confidence += 0.05
        elif dockerfile or _detect_language(root) != "unknown":
            classification = "CliOnly"
            confidence = 0.45
        else:
            classification = "Unknown"
            confidence = 0.3
        confidence = round(min(confidence, 0.95), 2)

        missing = []
        if not has_mcp:
            missing.append("mcp_server")
        if not has_openapi:
            missing.append("openapi_spec")
        if not has_rest and classification != "ApiOrMcp":
            missing.append("rest_api")
        if not routes:
            missing.append("tool_discovery")

        auth_hint = None
        if re.search(r"authorization: bearer|api[_-]?key|x-api-key",
                     readme, re.IGNORECASE):
            auth_hint = {"header": "Authorization", "env": name.upper().replace("-", "_") + "_TOKEN"}

        notes = []
        if not routes:
            notes.append("no explicit routes found; Creation stage should synthesize a wrapper")
        if classification == "CliOnly":
            notes.append("Class B: requires Docker sandbox wrapper (Execution phase 2)")

        _emit({
            "ok": True,
            "name": name,
            "description": description or f"Agent skill forged from {name}",
            "classification": classification,
            "confidence": confidence,
            "base_url_hint": base_url_hint,
            "auth_hint": auth_hint,
            "candidate_routes": dict(list(routes.items())[:30]),
            "surfaces": {
                "openapi": has_openapi, "mcp": has_mcp,
                "dockerfile": bool(dockerfile), "rest_hints": has_rest,
                "readme": bool(readme),
            },
            "missing_agent_surfaces": missing,
            "risk_signals": _risk_signals(readme, root),
            "nuclei": _nuclei_scan(root),
            "security_prescan": security_prescan,
            "clone_path": root,
            "language": _detect_language(root),
            "notes": notes,
        })
    except subprocess.TimeoutExpired:
        if tmp and os.path.isdir(tmp):
            shutil.rmtree(tmp, ignore_errors=True)
        _fail("clone or scan timed out")
    except Exception as e:  # never crash the Rust caller
        if tmp and os.path.isdir(tmp):
            shutil.rmtree(tmp, ignore_errors=True)
        _fail(f"analyzer exception: {type(e).__name__}: {e}")
    # NOTE: on success (or a security-prescan hard-fail, which also cleans up
    # inline before emitting) the clone under tmp/repo is deliberately left
    # on disk -- Stage 1c (boot_and_discover.py) needs the real checkout to
    # boot the app. Caller (skillforge.rs) owns cleanup after the pipeline
    # finishes with this run.


if __name__ == "__main__":
    main()
