"""
Constitutional layer for Ògún (Python) tool execution.

ToolConstitution defines the constitutional envelope for one tool — what it is
allowed to do, what it explicitly forbids, and which Hermetic principles it
is accountable to. ConstitutionalExecutor wraps OgunExecutor and runs every
tool invocation through constitutional pre-flight and post-flight checks.

Violations are fed to Ọ̀ṣun (SOMA memory) as learning events so future
calls benefit from the agent's constitutional history.
"""
from __future__ import annotations

import logging
import re
from dataclasses import dataclass, field
from typing import Any

from executor import OgunExecutor, PrivacyMode

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# ToolConstitution
# ---------------------------------------------------------------------------

@dataclass
class ToolConstitution:
    """Constitutional envelope for a single tool."""
    tool_name: str
    # Which Hermetic principles this tool is accountable to
    principles: list[str] = field(default_factory=lambda: [
        "Mentalism", "Correspondence", "Vibration",
        "Polarity", "Rhythm", "CauseAndEffect", "Gender",
    ])
    # Regexes that, if matched in intent+params, block the call
    forbidden_patterns: list[str] = field(default_factory=list)
    # Regexes that, if matched in the output, flag a post-call violation
    output_violation_patterns: list[str] = field(default_factory=list)
    # Minimum alignment score (0.0–1.0) required to allow the call
    min_alignment_score: float = 0.40
    # Whether this tool is allowed in incognito / private mode
    allow_in_private: bool = True
    # Human-readable rationale for this tool's constitutional constraints
    rationale: str = ""


# Default constitutions for built-in Ògún tools
DEFAULT_CONSTITUTIONS: dict[str, ToolConstitution] = {
    "bash": ToolConstitution(
        tool_name="bash",
        forbidden_patterns=[
            r"rm\s+-rf\s+/",          # recursive root delete
            r":(){ :|:& };:",          # fork bomb
            r"dd\s+if=/dev/zero",      # disk wipe
            r"mkfs\.",                 # filesystem format
            r"curl\s+.*\|\s*sh",       # curl-pipe-to-shell
        ],
        output_violation_patterns=[
            r"password\s*=",
            r"token\s*=",
            r"BEGIN\s+PRIVATE\s+KEY",
        ],
        min_alignment_score=0.50,
        rationale="bash is the highest-risk tool — extra constitutional scrutiny applied.",
    ),
    "read_file": ToolConstitution(
        tool_name="read_file",
        forbidden_patterns=[
            r"/etc/shadow",
            r"/proc/self/mem",
        ],
        min_alignment_score=0.35,
        rationale="File reads are low-risk but must not access credential stores.",
    ),
    "write_file": ToolConstitution(
        tool_name="write_file",
        forbidden_patterns=[
            r"/etc/",
            r"/proc/",
            r"/sys/",
        ],
        min_alignment_score=0.50,
        rationale="File writes require higher alignment — irreversible side effects.",
    ),
    "http_request": ToolConstitution(
        tool_name="http_request",
        forbidden_patterns=[
            r"169\.254\.169\.254",     # AWS metadata service
            r"metadata\.google\.internal",
        ],
        min_alignment_score=0.40,
        allow_in_private=False,
        rationale="HTTP requests must not reach cloud metadata endpoints.",
    ),
}


# ---------------------------------------------------------------------------
# ConstitutionalViolation
# ---------------------------------------------------------------------------

@dataclass
class ConstitutionalViolation:
    """A detected constitutional violation during tool execution."""
    tool_name: str
    phase: str          # "pre" | "post"
    principle: str
    pattern: str
    matched_text: str
    severity: str       # "warn" | "block"


# ---------------------------------------------------------------------------
# ConstitutionalExecutor
# ---------------------------------------------------------------------------

class ConstitutionalExecutor:
    """
    Wraps OgunExecutor with constitutional pre-flight and post-flight checks.

    Every tool invocation is evaluated against its ToolConstitution before
    execution. Violations are logged and fed to Ọ̀ṣun SOMA as learning events.
    Block-severity violations abort the call entirely.
    """

    def __init__(
        self,
        agent_id: str,
        tier: int = 0,
        privacy_mode: PrivacyMode = PrivacyMode.PUBLIC,
        constitutions: dict[str, ToolConstitution] | None = None,
        osun_callback: Any = None,
    ) -> None:
        self._executor = OgunExecutor(agent_id, tier, privacy_mode)
        self._constitutions = constitutions or DEFAULT_CONSTITUTIONS
        self._osun_callback = osun_callback  # callable(violation) or None
        self.violations: list[ConstitutionalViolation] = []

    def execute(
        self,
        tool_name: str,
        params: dict[str, Any],
        intent: str = "",
        provider: str = "webllm",
    ) -> dict[str, Any]:
        """
        Execute a tool through the constitutional envelope.

        Pre-flight: check intent + params against forbidden patterns.
        Execution: delegate to OgunExecutor.
        Post-flight: scan output for violation patterns.
        """
        constitution = self._constitutions.get(tool_name)

        # Pre-flight constitutional check
        if constitution is not None:
            block = self._preflight(intent, params, constitution)
            if block:
                return {
                    "error": f"constitutional_block: {block.pattern}",
                    "principle": block.principle,
                    "phase": "pre",
                    "tool": tool_name,
                }

        # Delegate to core executor
        result = self._executor.execute_tool(tool_name, params, provider)

        # Post-flight output scan
        if constitution is not None and "output" in result:
            warn = self._postflight(str(result.get("output", "")), tool_name, constitution)
            if warn:
                result["constitutional_warning"] = warn.pattern
                result["violated_principle"] = warn.principle

        return result

    # -----------------------------------------------------------------------

    def _preflight(
        self,
        intent: str,
        params: dict[str, Any],
        constitution: ToolConstitution,
    ) -> ConstitutionalViolation | None:
        """Check intent + params against forbidden patterns. Returns first block or None."""
        combined = (intent + " " + str(params)).lower()

        for pattern in constitution.forbidden_patterns:
            if re.search(pattern, combined, re.IGNORECASE):
                violation = ConstitutionalViolation(
                    tool_name=constitution.tool_name,
                    phase="pre",
                    principle="CauseAndEffect",
                    pattern=pattern,
                    matched_text=combined[:200],
                    severity="block",
                )
                self._record_violation(violation)
                return violation

        return None

    def _postflight(
        self,
        output: str,
        tool_name: str,
        constitution: ToolConstitution,
    ) -> ConstitutionalViolation | None:
        """Scan tool output for violation patterns. Returns first match or None."""
        for pattern in constitution.output_violation_patterns:
            if re.search(pattern, output, re.IGNORECASE):
                violation = ConstitutionalViolation(
                    tool_name=tool_name,
                    phase="post",
                    principle="Correspondence",
                    pattern=pattern,
                    matched_text=output[:200],
                    severity="warn",
                )
                self._record_violation(violation)
                return violation

        return None

    def _record_violation(self, violation: ConstitutionalViolation) -> None:
        self.violations.append(violation)
        logger.warning(
            "[ConstitutionalExecutor] %s violation in %s (%s): matched '%s'",
            violation.severity,
            violation.tool_name,
            violation.phase,
            violation.pattern,
        )
        if self._osun_callback is not None:
            try:
                self._osun_callback(violation)
            except Exception as exc:  # noqa: BLE001
                logger.debug("[ConstitutionalExecutor] osun_callback failed: %s", exc)

    def violation_count(self) -> int:
        return len(self.violations)

    def blocked_violations(self) -> list[ConstitutionalViolation]:
        return [v for v in self.violations if v.severity == "block"]
