#!/usr/bin/env python3
"""
boot_and_discover.py — SkillForge Stage 1c (mandatory, soft-fallback).

Given the already-security-cleared clone (Stage 0.5 must have passed --
this script never runs against anything that hasn't cleared the hard gate),
heuristically builds + boots the repo in an isolated, loopback-only process,
then drives it with Playwright (playwright_discover.js) to capture real
network requests, and brute-forces unlinked paths with gobuster. Routes
found this way are strictly additive to whatever static analysis (Stage 1)
already found -- this never removes a route, only enriches candidate_routes
with what static analysis structurally cannot see (JS-rendered routes,
undocumented paths).

Soft fallback: if boot fails (language undetectable, install fails, nothing
ever answers on the port), this exits ok=True, booted=False -- the pipeline
proceeds on static-only routes. The caller (skillforge.rs) is responsible
for stamping the skill fully_forged=False in that case; this script never
lies about what it accomplished.

Usage: boot_and_discover.py --dir <clone_path> --language <lang> [--port N] [--boot-timeout N]
Emits one JSON object on stdout. Never raises.
"""
import argparse
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import time

NODE_JS = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'playwright_discover.js')
PARROT_CONTAINER = os.environ.get('SKILLFORGE_PARROT_CONTAINER', 'ares-parrot')


def _port_open(port, host='127.0.0.1'):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.settimeout(0.5)
        return s.connect_ex((host, port)) == 0


def _free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]


def _boot_plan(root, language, port):
    """Returns (install_cmd|None, run_cmd|None, env) heuristics per
    language. None, None means we don't know how to boot this repo --
    soft-fallback territory, not a bug."""
    env = os.environ.copy()
    env['PORT'] = str(port)
    env['HOST'] = '0.0.0.0'
    if language in ('javascript', 'typescript', 'node'):
        if os.path.isfile(os.path.join(root, 'package.json')):
            try:
                pkg = json.load(open(os.path.join(root, 'package.json')))
            except Exception:
                pkg = {}
            scripts = pkg.get('scripts', {})
            run = ['npm', 'start'] if 'start' in scripts else None
            if not run:
                for entry in ('index.js', 'server.js', 'app.js', pkg.get('main', '')):
                    if entry and os.path.isfile(os.path.join(root, entry)):
                        run = ['node', entry]
                        break
            return (['npm', 'install', '--no-audit', '--no-fund', '--omit=dev'], run, env)
    if language == 'python':
        req = os.path.join(root, 'requirements.txt')
        entry = None
        for cand in ('app.py', 'main.py', 'server.py', 'manage.py'):
            if os.path.isfile(os.path.join(root, cand)):
                entry = cand
                break
        install = ['pip', 'install', '-q', '-r', req] if os.path.isfile(req) else None
        run = ['python3', entry] if entry else None
        return (install, run, env)
    if language == 'go':
        if os.path.isfile(os.path.join(root, 'go.mod')):
            return (['go', 'build', '-o', '/tmp/skillforge_go_boot', '.'], ['/tmp/skillforge_go_boot'], env)
    return (None, None, env)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--dir', required=True)
    ap.add_argument('--language', default='unknown')
    ap.add_argument('--base-url-hint', default=None)
    ap.add_argument('--boot-timeout', type=int, default=180)
    ap.add_argument('--ready-wait', type=int, default=20)
    args = ap.parse_args()

    result = {
        'ok': True,             # this script's own execution succeeded
        'booted': False,        # did we get the target app running
        'discovered_routes': {},
        'ui_only': False,
        'gobuster_paths': [],
        'playwright': None,
        'notes': [],
    }

    port = None
    if args.base_url_hint:
        m = re.search(r':(\d+)', args.base_url_hint)
        if m:
            port = int(m.group(1))
    if not port:
        port = _free_port()

    install_cmd, run_cmd, env = _boot_plan(args.dir, args.language, port)
    if not run_cmd:
        result['notes'].append(f'no boot heuristic for language={args.language}; static-only fallback')
        print(json.dumps(result))
        return

    proc = None
    try:
        if install_cmd:
            try:
                subprocess.run(install_cmd, cwd=args.dir, env=env, capture_output=True,
                                text=True, timeout=args.boot_timeout)
            except Exception as e:
                result['notes'].append(f'install step failed: {e}; attempting boot anyway')

        try:
            proc = subprocess.Popen(run_cmd, cwd=args.dir, env=env,
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     preexec_fn=os.setsid)
        except Exception as e:
            result['notes'].append(f'boot failed to start: {e}')
            print(json.dumps(result))
            return

        ready = False
        for _ in range(args.ready_wait):
            if _port_open(port):
                ready = True
                break
            if proc.poll() is not None:
                break
            time.sleep(1)

        if not ready:
            result['notes'].append('app never became reachable on the expected port; static-only fallback')
            print(json.dumps(result))
            return

        result['booted'] = True
        base = f'http://127.0.0.1:{port}'

        # Playwright: click-through + network capture
        if shutil.which('node'):
            try:
                pw = subprocess.run(
                    ['node', NODE_JS, '--target', base, '--max-clicks', '15', '--timeout-ms', '15000'],
                    capture_output=True, text=True, timeout=60,
                )
                pw_json = json.loads(pw.stdout.strip().splitlines()[-1]) if pw.stdout.strip() else {}
                result['playwright'] = pw_json
                result['ui_only'] = pw_json.get('ui_only', False)
                for i, req in enumerate(pw_json.get('requests', [])):
                    if req.get('resource_type') == 'document':
                        continue
                    name = f"discovered_{i}_{req['method'].lower()}"
                    path = re.sub(r'^https?://[^/]+', '', req['url']) or '/'
                    result['discovered_routes'][name] = f"{req['method']} {path}"
            except Exception as e:
                result['notes'].append(f'playwright discovery failed: {e}')

        # gobuster: unlinked paths, via the ares-parrot toolbox container
        if shutil.which('docker'):
            try:
                gw = subprocess.run(
                    ['docker', 'network', 'inspect', 'bridge', '--format',
                     '{{(index .IPAM.Config 0).Gateway}}'],
                    capture_output=True, text=True, timeout=10,
                ).stdout.strip() or '172.17.0.1'
                gb = subprocess.run(
                    ['docker', 'exec', PARROT_CONTAINER, 'gobuster', 'dir',
                     '-u', f'http://{gw}:{port}', '-w',
                     '/usr/share/wordlists/dirb/common.txt', '-q', '-t', '20'],
                    capture_output=True, text=True, timeout=45,
                )
                for line in gb.stdout.splitlines():
                    if 'Status: 200' in line or 'Status: 301' in line:
                        path = line.split()[0]
                        result['gobuster_paths'].append(path)
                        name = f"discovered_gobuster_{len(result['gobuster_paths'])}"
                        result['discovered_routes'][name] = f'GET {path}'
            except Exception as e:
                result['notes'].append(f'gobuster failed: {e}')
    finally:
        if proc is not None:
            try:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
            except Exception:
                pass

    print(json.dumps(result))


if __name__ == '__main__':
    main()
