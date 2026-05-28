"""
Code runner tool — executes Python code in an isolated subprocess.
Tier 2 (Creator): requires elevated trust.

Safety: runs in a temporary directory with a hard timeout. Full kernel-level
sandboxing (seccomp, namespaces) is left to the deployment layer.
"""

import json
import os
import subprocess
import sys
import tempfile
import textwrap

# Hard limits
TIMEOUT_SECONDS = 10
MAX_OUTPUT_BYTES = 16_384
MAX_CODE_BYTES = 32_768


def code_runner(params: str) -> str:
    """
    Execute Python code and return stdout + stderr.

    Params JSON: {"code": "...", "timeout": 10, "stdin": "..."}
    Or plain Python code string.
    """
    code, timeout, stdin_data = _parse_params(params)

    if len(code.encode()) > MAX_CODE_BYTES:
        raise ValueError(f"code exceeds {MAX_CODE_BYTES} byte limit")

    with tempfile.TemporaryDirectory(prefix="omokoda_runner_") as tmpdir:
        script_path = os.path.join(tmpdir, "script.py")
        with open(script_path, "w") as f:
            f.write(textwrap.dedent(code))

        try:
            result = subprocess.run(
                [sys.executable, script_path],
                cwd=tmpdir,
                input=stdin_data,
                capture_output=True,
                timeout=timeout,
                text=True,
                env={
                    "PATH": os.environ.get("PATH", ""),
                    "PYTHONPATH": "",
                    "HOME": tmpdir,
                    "TMPDIR": tmpdir,
                },
            )
        except subprocess.TimeoutExpired:
            raise TimeoutError(f"code execution timed out after {timeout}s")

    stdout = result.stdout[:MAX_OUTPUT_BYTES]
    stderr = result.stderr[:MAX_OUTPUT_BYTES]

    output: dict = {
        "exit_code": result.returncode,
        "stdout": stdout,
        "stderr": stderr,
    }

    if result.returncode != 0:
        raise RuntimeError(json.dumps(output))

    return json.dumps(output)


def _parse_params(params: str) -> tuple[str, int, str | None]:
    params = params.strip()
    if params.startswith("{"):
        try:
            obj = json.loads(params)
            code = obj.get("code", "")
            timeout = min(int(obj.get("timeout", TIMEOUT_SECONDS)), TIMEOUT_SECONDS)
            stdin_data = obj.get("stdin")
            return code, timeout, stdin_data
        except (json.JSONDecodeError, ValueError):
            pass
    return params, TIMEOUT_SECONDS, None
