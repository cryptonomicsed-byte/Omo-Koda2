#!/usr/bin/env python3
"""yt_harvest.py — SkillForge YouTube intake: harvest + frankenstein classify.

Given a YouTube URL (single video OR channel/playlist), pull the video
description(s) with NO API key (yt-dlp if on PATH, else plain HTTP GET of the
public watch page), extract every repo link (github.com / gitlab.com / any
git host), shallow-clone each one, and classify it for *frankenstein*
potential — the "can this bolt onto other repos" profile:

  category        purpose bucket (recon/instrumentation/network/exploit/...)
  stack           top languages + framework hints
  android_capable Android project (manifest/gradle) — runs on the Fold4/emulators
  termux_runnable runs under Termux (python/node/rust/go + build hints)
  license, size, description, entry_points

Emits ONE JSON object on stdout:
  {
    "ok": true,
    "source_url": "...",
    "kind": "video" | "playlist" | "channel",
    "videos": [{"id","title","url"}],
    "repos": [
      {
        "url", "host", "owner", "repo",
        "video_ids": [...],          # which videos linked it
        "category", "stack", "android_capable", "termux_runnable",
        "license", "size_kb", "description", "entry_points",
        "clone_ok": true|false, "skip_reason": null|"..."
      }, ...
    ],
    "harvest_error": null | "..."
  }

Exit 0 on success (even with zero repos — check `repos`), non-zero only on a
total failure (bad URL, no description anywhere).

Usage: yt_harvest.py <youtube-url> [--max-videos N] [--max-clone-kb N]
Env:   YT_HARVEST_MAX_VIDEOS   cap videos scanned for channel/playlist (default 10)
       YT_HARVEST_MAX_CLONE_KB cap per-repo shallow clone size (default 20480)
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request
from collections import Counter
from pathlib import Path
from typing import Optional

MAX_VIDEOS = int(os.environ.get("YT_HARVEST_MAX_VIDEOS", "10"))
MAX_CLONE_KB = int(os.environ.get("YT_HARVEST_MAX_CLONE_KB", "20480"))
CLONE_TIMEOUT_S = 90
FETCH_TIMEOUT_S = 30

# ── URL parsing ──────────────────────────────────────────────────────────

_VIDEO_ID_RE = re.compile(
    r"(?:youtube\.com/(?:watch\?v=|shorts/|embed/|live/)|youtu\.be/)([A-Za-z0-9_-]{11})"
)
_CHANNEL_RE = re.compile(r"youtube\.com/(?:@|c/|channel/|user/)([A-Za-z0-9_-]+)")
_PLAYLIST_RE = re.compile(r"youtube\.com/playlist\?list=([A-Za-z0-9_-]+)")

# github/gitlab/gitea/bitbucket + any <host>/owner/repo pattern. Owner/repo
# are the GitHub-legal charset; repo names may carry a .git suffix (kept).
_REPO_RE = re.compile(
    r"https?://([A-Za-z0-9.-]+(?:\.(?:com|org|io|dev|net))?)/([A-Za-z0-9][A-Za-z0-9_.-]*)/([A-Za-z0-9][A-Za-z0-9_.-]*?)(?:\.git)?(?:/|$|\s|[.,;:!?)\]}>\"'])"
)
_TRAILING_PUNCT_RE = re.compile(r"[.,;:!?)\]}'\"`]+$")
_SKIP_HOSTS = {"www.youtube.com", "youtu.be", "youtube.com", "m.youtube.com",
               "www.google.com", "raw.githubusercontent.com"}

# ── Purpose classification (keyword → category) ───────────────────────────

_CATEGORY_RULES: list[tuple[str, tuple[str, ...]]] = [
    ("recon",        ("nmap", "recon", "scanner", "subdomain", "osint", "fingerprint",
                      "shodan", "port scan", "network scan", "masscan", "enumeration")),
    ("exploitation", ("exploit", "metasploit", "payload", "cve", "rce", "shellcode",
                      "buffer overflow", "pwn", "privilege escalation", "reverse shell")),
    ("instrumentation", ("frida", "hook", "inject", "instrument", "trace", "dbi",
                         "runtime manipulation", "xposed", "magisk", "zygisk")),
    ("network",      ("proxy", "sniff", "mitm", "tcp", "udp", "websocket", "http client",
                      "vpn", "tunnel", "packet", "pcap", "capture", "ddos", "botnet")),
    ("crypto",       ("wallet", "solana", "ethereum", "bitcoin", "token", "defi",
                      "smart contract", "nft", "mnemonic", "bip39", "private key")),
    ("ui",           ("react", "vue", "frontend", "dashboard", "web ui", "tui",
                      "terminal ui", "gui", "android app", "flutter", "next.js")),
    ("automation",   ("automation", "bot", "cron", "scrape", "crawler", "selenium",
                      "playwright", "workflow", "pipeline", "ci/cd", "api client")),
    ("evasion",      ("stealth", "evasion", "anti-detect", "camoufox", "proxy rotation",
                      "fingerprint spoof", "captcha", "bypass")),
    ("llm",          ("llm", "gpt", "agent", "prompt", "rag", "fine-tun", "tokenizer",
                      "embedding", "langchain", "model")),
    ("mobile",       ("android", "termux", "apk", "adb", "mobile", "smali", "dex")),
]

# Stack hints: marker file → (language, framework tag)
_STACK_HINTS: list[tuple[str, tuple[str, ...]]] = [
    ("Cargo.toml",      ("rust",)),
    ("package.json",    ("javascript", "node")),
    ("go.mod",          ("go",)),
    ("pyproject.toml",  ("python",)),
    ("requirements.txt",("python",)),
    ("setup.py",        ("python",)),
    ("pom.xml",         ("java",)),
    ("build.gradle",    ("java", "kotlin")),
    ("build.gradle.kts",("kotlin",)),
    ("Gemfile",         ("ruby",)),
    ("composer.json",   ("php",)),
    ("CMakeLists.txt",  ("c", "cpp")),
    ("Makefile",        ("c",)),
    ("*.csproj",        ("csharp",)),
    ("Cargo.lock",      ("rust",)),
]

_EXT_LANG: dict[str, str] = {
    ".py": "python", ".js": "javascript", ".ts": "typescript", ".tsx": "typescript",
    ".rs": "rust", ".go": "go", ".c": "c", ".h": "c", ".cpp": "cpp", ".hpp": "cpp",
    ".java": "java", ".kt": "kotlin", ".rb": "ruby", ".php": "php", ".cs": "csharp",
    ".swift": "swift", ".sh": "shell", ".lua": "lua", ".vue": "vue", ".sol": "solidity",
    ".m": "matlab", ".jl": "julia", ".clj": "clojure", ".ex": "elixir",
}


def _classify_category(repo_name: str, readme_text: str, files: list[str]) -> str:
    hay = f"{repo_name} {readme_text[:8000].lower()} {' '.join(files[:200]).lower()}"
    scores: Counter[str] = Counter()
    for cat, kws in _CATEGORY_RULES:
        for kw in kws:
            if kw in hay:
                scores[cat] += 1
    if not scores:
        return "general"
    return scores.most_common(1)[0][0]


def _classify_stack(root: Path, files: list[str]) -> list[str]:
    hints: list[str] = []
    for marker, langs in _STACK_HINTS:
        if marker.startswith("*"):
            if any(f.endswith(marker[1:]) for f in files):
                hints.extend(langs)
        elif (root / marker).exists():
            hints.extend(langs)
    # extension histogram on top-level + shallow walk (skip .git, node_modules)
    ext_counts: Counter[str] = Counter()
    for f in files:
        ext = Path(f).suffix.lower()
        if ext in _EXT_LANG:
            ext_counts[_EXT_LANG[ext]] += 1
    top = [lang for lang, _ in ext_counts.most_common(3)]
    merged: list[str] = []
    for lang in hints + top:
        if lang not in merged:
            merged.append(lang)
    return merged[:4]


def _android_capable(root: Path, files: list[str]) -> bool:
    return any(
        (root / f).exists()
        for f in ("AndroidManifest.xml", "build.gradle", "build.gradle.kts",
                  "settings.gradle", "settings.gradle.kts", "gradlew")
    ) or any("android" in f.lower() for f in files[:400])


def _termux_runnable(root: Path, files: list[str]) -> bool:
    """Best-effort: does this repo look like it runs under Termux (python /
    node / go / rust / clang toolchains all exist in Termux)?"""
    markers = ("install.sh", "setup.sh", "Makefile", "requirements.txt",
               "package.json", "Cargo.toml", "go.mod", "pyproject.toml", "setup.py")
    if not any((root / m).exists() for m in markers):
        return False
    # Heavy native/system deps (X11, systemd, kernel modules) → not Termux.
    hay = " ".join(f.lower() for f in files[:300])
    for blocker in ("systemd", "/usr/lib/", "apt-get install", "dnf install",
                    "yum install", "x11", ".desktop", "kernel module", "dkms"):
        if blocker in hay:
            return False
    return True


def _license_name(root: Path) -> str:
    for cand in ("LICENSE", "LICENSE.md", "LICENSE.txt", "LICENSE-MIT",
                 "LICENSE-APACHE", "COPYING", "UNLICENSE"):
        p = root / cand
        if p.exists():
            try:
                first = p.read_text(errors="replace")[:200].strip()
                for line in first.splitlines():
                    line = line.strip()
                    if line and not line.startswith(("#", "/*", "*", "//")):
                        if re.search(r"MIT|Apache|GPL|BSD|MPL|AGPL|LGPL|ISC|Unlicense", line, re.I):
                            return line[:80]
                        return cand
            except OSError:
                return cand
    return "unknown"


def _walk_files(root: Path) -> list[str]:
    out: list[str] = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames
                       if d not in (".git", "node_modules", ".venv", "venv",
                                    "target", "dist", "build", "__pycache__", ".cache")]
        for fn in filenames:
            p = Path(dirpath) / fn
            try:
                rel = str(p.relative_to(root))
            except ValueError:
                rel = str(p)
            out.append(rel)
            if len(out) > 4000:
                return out
    return out


def _clone_and_classify(repo_url: str) -> dict:
    """Shallow-clone one repo and emit its frankenstein profile."""
    entry: dict = {
        "url": repo_url, "clone_ok": False, "skip_reason": None,
        "category": None, "stack": [], "android_capable": False,
        "termux_runnable": False, "license": None, "size_kb": 0,
        "description": None, "entry_points": [],
    }
    with tempfile.TemporaryDirectory(prefix="ytharvest-") as td:
        dest = Path(td) / "repo"
        try:
            proc = subprocess.run(
                ["git", "clone", "--depth", "1", "--quiet", repo_url, str(dest)],
                capture_output=True, text=True, timeout=CLONE_TIMEOUT_S,
            )
        except subprocess.TimeoutExpired:
            entry["skip_reason"] = "clone timeout"
            return entry
        except OSError as e:
            entry["skip_reason"] = f"git unavailable: {e}"
            return entry
        if proc.returncode != 0:
            entry["skip_reason"] = (proc.stderr or "clone failed").strip()[:200]
            return entry

        try:
            kb = int(subprocess.run(["du", "-sk", str(dest)],
                                    capture_output=True, text=True, timeout=10)
                     .stdout.split()[0])
        except Exception:
            kb = 0
        entry["size_kb"] = kb
        if kb > MAX_CLONE_KB:
            entry["skip_reason"] = f"too large ({kb} kB > {MAX_CLONE_KB} kB cap)"
            return entry

        files = _walk_files(dest)
        readme = ""
        for cand in ("README.md", "README", "readme.md", "README.txt", "Readme.md"):
            p = dest / cand
            if p.exists():
                try:
                    readme = p.read_text(errors="replace")[:12000]
                except OSError:
                    readme = ""
                break
        host, _, path = repo_url.replace("https://", "").partition("/")
        owner_repo = path.rstrip("/").removesuffix(".git")
        owner, _, repo = owner_repo.partition("/")

        entry.update({
            "host": host, "owner": owner, "repo": repo,
            "clone_ok": True,
            "category": _classify_category(repo, readme, files),
            "stack": _classify_stack(dest, files),
            "android_capable": _android_capable(dest, files),
            "termux_runnable": _termux_runnable(dest, files),
            "license": _license_name(dest),
            "description": re.sub(r"\s+", " ", readme)[:240] if readme else None,
            "entry_points": [f for f in files[:400]
                             if re.search(r"(^|/)(main|app|cli|index|run)[._-]?[a-z]*\.(py|js|ts|rs|go|sh)$",
                                          f, re.I) or f in ("Makefile", "install.sh", "setup.sh")][:8],
        })
    return entry


# ── Description fetch (no API key) ────────────────────────────────────────

def _find_yt_dlp() -> Optional[str]:
    cand = Path(sys.executable).parent / "yt-dlp"
    if cand.exists():
        return str(cand)
    return shutil.which("yt-dlp")


_PLAYER_RESPONSE_RE = re.compile(
    r"ytInitialPlayerResponse\s*=\s*(\{.*?\})\s*;\s*(?:var |</script>)"
)

# ── Web-search description fallback (no key, scrape-friendly endpoints) ───
# Google/Bing/DDG SERPs index YouTube descriptions; snippets often carry the
# full description text including repo links. This is the LAST fetch fallback
# (after yt-dlp, HTTP page, Invidious) and is most reliable from residential
# IPs — datacenter IPs frequently get bot-walled. It also doubles as the
# "find the repos even when the video page itself is blocked" path.

_SEARCH_UA = ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
              "(KHTML, like Gecko) Chrome/124.0 Safari/537.36")


def _search_query_urls(video_id: str, title: str) -> list[str]:
    """Build a small list of SERP URLs to try, most scrape-friendly first."""
    queries = [f'"{video_id}"']
    if title:
        # Title words only (drop stopwords) keep the query tight.
        words = [w for w in re.findall(r"[A-Za-z0-9]+", title)
                 if w.lower() not in {"the", "and", "for", "with", "in", "of",
                                      "a", "an", "to", "on", "2026"}][:6]
        if len(words) >= 3:
            queries.append(" ".join(words))
    urls: list[str] = []
    for q in queries:
        qq = urllib.parse.quote_plus(q)
        urls.append(f"https://lite.duckduckgo.com/lite/?q={qq}")
        urls.append(f"https://html.duckduckgo.com/html/?q={qq}")
        urls.append(f"https://www.bing.com/search?q={qq}")
        urls.append(f"https://www.google.com/search?q={qq}&gbv=1")
    return urls


def _try_web_search(video_id: str, title: str) -> Optional[tuple[str, str]]:
    """Search SERPs for the video's description text (title + repo links).

    Returns (title, description) when a SERP snippet looks like a YouTube
    description (contains the video ID or ≥2 github/gitlab/gitea links).
    """
    import urllib.request
    import urllib.parse
    for url in _search_query_urls(video_id, title):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": _SEARCH_UA})
            with urllib.request.urlopen(req, timeout=12) as resp:
                html = resp.read().decode("utf-8", errors="replace")
        except Exception:
            continue
        if not html:
            continue
        # A usable SERP either mentions the video ID or carries repo links.
        repo_hits = _REPO_URL_RE.findall(html)
        if not (video_id in html or len(repo_hits) >= 2):
            continue
        text = re.sub(r"<[^>]+>", " ", html)
        text = re.sub(r"\s+", " ", text)
        # Find the chunk around the first repo link — usually the description.
        for m in _REPO_URL_RE.finditer(text):
            start = max(0, m.start() - 600)
            snippet = text[start:m.end() + 400]
            if video_id in snippet or len(_REPO_URL_RE.findall(snippet)) >= 2:
                return title or "", snippet
    return None


def _fetch_video(video_id: str, title: Optional[str] = None) -> Optional[dict]:
    """Return {id,title,description} or None. yt-dlp → HTTP page → Invidious
    → web-search SERP (last two most reliable from residential IPs)."""
    binary = _find_yt_dlp()
    if binary:
        try:
            proc = subprocess.run(
                [binary, "--dump-json", "--skip-download", "--no-warnings",
                 "--no-playlist", f"https://www.youtube.com/watch?v={video_id}"],
                capture_output=True, text=True, timeout=FETCH_TIMEOUT_S,
            )
            if proc.returncode == 0:
                data = json.loads(proc.stdout)
                return {"id": video_id, "title": data.get("title", ""),
                        "description": data.get("description", "") or ""}
        except Exception:
            pass
    try:
        import urllib.request
        req = urllib.request.Request(
            f"https://www.youtube.com/watch?v={video_id}",
            headers={"User-Agent": _SEARCH_UA,
                     "Accept-Language": "en-US,en;q=0.9"},
        )
        with urllib.request.urlopen(req, timeout=FETCH_TIMEOUT_S) as resp:
            html = resp.read().decode("utf-8", errors="replace")
        m = _PLAYER_RESPONSE_RE.search(html)
        if m:
            data = json.loads(m.group(1))
            details = data.get("videoDetails", {}) or {}
            return {"id": video_id, "title": details.get("title", ""),
                    "description": details.get("shortDescription", "") or ""}
    except Exception:
        pass
    # Invidious API pool (key-free metadata; many instances are dead — rotate).
    for inst in ("inv.nadeko.net", "yewtu.be", "invidious.nerdvpn.de",
                 "iv.melmac.space", "inv.tux.pizza", "invidious.f5.si"):
        try:
            import urllib.request
            req = urllib.request.Request(
                f"https://{inst}/api/v1/videos/{video_id}?fields=title,description",
                headers={"User-Agent": _SEARCH_UA},
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                data = json.loads(resp.read().decode("utf-8", errors="replace"))
            if isinstance(data, dict) and data.get("description"):
                return {"id": video_id, "title": data.get("title") or title or "",
                        "description": data["description"]}
        except Exception:
            continue
    # Web-search SERP fallback (user-requested path; best from residential IP).
    return _try_web_search(video_id, title or "")


def _resolve_urls(url: str) -> tuple[list[dict], str]:
    """Map the input URL to the list of concrete video IDs to scan.

    Returns (videos, kind) where kind ∈ {video, playlist, channel, unknown}.
    Channel/playlist URLs enumerate up to MAX_VIDEOS videos via yt-dlp
    (--flat-playlist gives ids cheaply; descriptions are fetched per video
    by _fetch_video). Single video URLs resolve directly.
    """
    m = _VIDEO_ID_RE.search(url)
    if m:
        return [{"id": m.group(1), "title": None}], "video"

    is_playlist = _PLAYLIST_RE.search(url)
    is_channel = _CHANNEL_RE.search(url)
    if not (is_playlist or is_channel):
        return [], "unknown"

    binary = _find_yt_dlp()
    if not binary:
        # Without yt-dlp we cannot enumerate channel/playlist members.
        return [], ("playlist" if is_playlist else "channel")

    cmd = [binary, "--flat-playlist", "--print", "%(id)s|%(title)s",
           "--playlist-items", f"1-{MAX_VIDEOS}",
           "--no-warnings", url]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except Exception:
        return [], ("playlist" if is_playlist else "channel")
    videos: list[dict] = []
    if proc.returncode == 0:
        for line in proc.stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            vid, _, title = line.partition("|")
            if re.fullmatch(r"[A-Za-z0-9_-]{11}", vid):
                videos.append({"id": vid, "title": title or None})
    return videos, ("playlist" if is_playlist else "channel")


def main() -> int:
    ap = argparse.ArgumentParser(description="SkillForge YouTube harvest + frankenstein classify")
    ap.add_argument("url")
    ap.add_argument("--description", help="skip the network fetch and use this pasted description")
    ap.add_argument("--title", help="video title (used with --description)")
    args = ap.parse_args()

    # Paste-mode: no network fetch at all — caller supplied the description
    # (fetched from a residential network, e.g. the Mac panel). Works from
    # any IP, including bot-walled datacenter hosts.
    if args.description:
        vid = _VIDEO_ID_RE.search(args.url)
        video_id = vid.group(1) if vid else "pasted"
        fetched = [{"id": video_id,
                    "title": args.title or f"pasted:{video_id}",
                    "description": args.description}]
        kind = "video"
        videos = [{"id": video_id, "title": args.title}]
        harvest_errors: list[str] = []
        return _emit(args.url, kind, videos, fetched, harvest_errors)

    videos, kind = _resolve_urls(args.url)
    if not videos:
        print(json.dumps({
            "ok": False, "source_url": args.url, "kind": kind,
            "error": "could not resolve any videos from URL "
                     f"(kind={kind}); needs yt-dlp for channel/playlist URLs",
            "videos": [], "repos": [],
        }))
        return 2

    fetched: list[dict] = []
    harvest_errors: list[str] = []
    for v in videos:
        info = _fetch_video(v["id"])
        if info is None:
            harvest_errors.append(v["id"])
            continue
        info["title"] = v.get("title") or info["title"]
        fetched.append(info)

    if not fetched:
        print(json.dumps({
            "ok": False, "source_url": args.url, "kind": kind,
            "error": f"failed to fetch any of {len(videos)} video description(s)",
            "videos": videos, "repos": [],
        }))
        return 2

    return _emit(args.url, kind, videos, fetched, harvest_errors)


def _emit(source_url: str, kind: str, videos: list[dict],
          fetched: list[dict], harvest_errors: list[str]) -> int:
    """Shared tail: extract repo URLs from fetched descriptions, clone +
    classify each, print the final JSON envelope. Used by both the network
    fetch path and paste-mode (--description)."""
    # Extract + dedupe repos across videos (order-preserving, first seen wins).
    seen: dict[str, dict] = {}
    for info in fetched:
        for m in _REPO_RE.finditer(info["description"] or ""):
            host = m.group(1).lower()
            if host in _SKIP_HOSTS:
                continue
            owner = m.group(2)
            repo = _TRAILING_PUNCT_RE.sub("", m.group(3))
            if not owner or not repo:
                continue
            repo = repo.removesuffix(".git")
            url = f"https://{host}/{owner}/{repo}"
            if url not in seen:
                seen[url] = {"url": url, "host": host, "owner": owner,
                             "repo": repo, "video_ids": []}
            if info["id"] not in seen[url]["video_ids"]:
                seen[url]["video_ids"].append(info["id"])

    repos: list[dict] = []
    for entry in seen.values():
        profile = _clone_and_classify(entry["url"])
        entry.update(profile)
        repos.append(entry)

    print(json.dumps({
        "ok": True, "source_url": source_url, "kind": kind,
        "videos": [{"id": v["id"], "title": v["title"]} for v in fetched],
        "repos": repos,
        "harvest_error": (f"could not fetch {len(harvest_errors)}/{len(videos)} "
                          f"video(s): {', '.join(harvest_errors)}")
        if harvest_errors else None,
    }, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
