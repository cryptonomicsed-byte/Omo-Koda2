"""
Ògún Executor — Ọmọ Kọ́dà Tool Execution Engine

The Steward (Èṣù) calls this executor to run tools inside the WASM sandbox
and route LLM requests based on the agent's privacy mode.

Privacy enforcement (non-negotiable):
- PUBLIC: any provider allowed
- PRIVATE / INCOGNITO: hard fail if provider is not WebLLM or Ollama
  Never silently reroute to an external provider.

Three primitives only: birth, think, act — no others.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

logger = logging.getLogger(__name__)

# --- Constants ---

LOCAL_PROVIDERS = frozenset({"webllm", "ollama"})

TIER_TOOLS: dict[int, frozenset[str]] = {
    0: frozenset({"web_search", "note_taking", "read_file", "glob", "grep"}),
    1: frozenset({"web_search", "note_taking", "read_file", "glob", "grep", "image_gen_basic"}),
    2: frozenset({"web_search", "note_taking", "read_file", "glob", "grep",
                  "image_gen_basic", "code_runner", "bash"}),
    3: frozenset({"web_search", "note_taking", "read_file", "glob", "grep",
                  "image_gen_basic", "code_runner", "bash", "data_analysis", "api_connect"}),
    4: frozenset({"web_search", "note_taking", "read_file", "glob", "grep",
                  "image_gen_basic", "code_runner", "bash", "data_analysis", "api_connect",
                  "agent_orchestration"}),
    5: frozenset({"web_search", "note_taking", "read_file", "glob", "grep",
                  "image_gen_basic", "code_runner", "bash", "data_analysis", "api_connect",
                  "agent_orchestration", "self_modification", "multi_agent_fabric"}),
}


class PrivacyMode(Enum):
    PUBLIC = "public"
    PRIVATE = "private"
    INCOGNITO = "incognito"


class ExecutionError(Exception):
    pass


class PrivacyViolation(ExecutionError):
    """Raised when a /private or INCOGNITO request is routed to an external provider."""


class TierViolation(ExecutionError):
    """Raised when a tool is not available at the agent's current tier."""


# --- Result types ---

@dataclass
class ToolResult:
    success: bool
    output: str
    synapse_burned: int
    tee_sealed: bool = False
    error: Optional[str] = None


# --- Executor ---

class ÒgúnExecutor:
    """
    Tool execution engine for the Ọmọ Kọ́dà Agent OS.

    Responsibilities:
    - Enforce tier-based tool access
    - Route LLM calls through the correct provider
    - Hard-fail on privacy violations (never silently reroute)
    - Compute and burn Synapse cost before execution
    """

    def __init__(
        self,
        tier: int,
        privacy_mode: PrivacyMode = PrivacyMode.PUBLIC,
        provider: str = "ollama",
    ) -> None:
        if tier not in range(6):
            raise ValueError(f"Invalid tier {tier}: must be 0–5")
        self.tier = tier
        self.privacy_mode = privacy_mode
        self.provider = provider.lower()

    # --- Public API (called by the Steward) ---

    def execute_tool(self, tool: str, params: dict) -> ToolResult:
        """Execute a tool after enforcing tier and Synapse constraints."""
        self._check_tier_allowed(tool)
        synapse_cost = self._compute_synapse_cost(tool)
        result = self._dispatch(tool, params)
        return ToolResult(
            success=result.success,
            output=result.output,
            synapse_burned=synapse_cost,
            tee_sealed=result.tee_sealed,
            error=result.error,
        )

    def execute_think(self, prompt: str, private: bool = False) -> ToolResult:
        """Route a think call to the appropriate provider."""
        if private or self.privacy_mode in (PrivacyMode.PRIVATE, PrivacyMode.INCOGNITO):
            self._enforce_local_provider()

        # In INCOGNITO mode: no logging of prompt or output
        log_prompt = "[redacted]" if self.privacy_mode == PrivacyMode.INCOGNITO else prompt

        logger.info("think: provider=%s prompt=%s", self.provider, log_prompt)

        # Stub: actual LLM call would go here
        return ToolResult(
            success=True,
            output=f"[think via {self.provider}]",
            synapse_burned=8,
        )

    # --- Internal ---

    def _check_tier_allowed(self, tool: str) -> None:
        allowed = TIER_TOOLS.get(self.tier, frozenset())
        if tool not in allowed:
            raise TierViolation(
                f"Tool '{tool}' is not available at tier {self.tier}. "
                f"Available tools: {sorted(allowed)}"
            )

    def _compute_synapse_cost(self, tool: str) -> int:
        base_costs: dict[str, int] = {
            "web_search": 40,
            "note_taking": 10,
            "read_file": 20,
            "glob": 15,
            "grep": 15,
            "image_gen_basic": 60,
            "code_runner": 80,
            "bash": 100,
            "data_analysis": 120,
            "api_connect": 80,
            "agent_orchestration": 200,
            "self_modification": 500,
            "multi_agent_fabric": 400,
        }
        return base_costs.get(tool, 50)

    def _enforce_local_provider(self) -> None:
        """Hard fail if the provider is not a local model runner."""
        if self.provider not in LOCAL_PROVIDERS:
            raise PrivacyViolation(
                f"Privacy violation: /private and INCOGNITO modes require a local provider "
                f"(webllm or ollama). Requested provider '{self.provider}' is external. "
                "Refusing to escalate. Use a local model or switch to PUBLIC mode."
            )

    def _dispatch(self, tool: str, params: dict) -> ToolResult:
        """Dispatch tool to the appropriate runtime stub."""
        # TEE stub for sensitive operations
        if self.privacy_mode in (PrivacyMode.PRIVATE, PrivacyMode.INCOGNITO):
            return self._tee_execute_stub(tool, params)
        return ToolResult(
            success=True,
            output=f"[{tool}] executed with params={params}",
            synapse_burned=0,
        )

    def _tee_execute_stub(self, tool: str, params: dict) -> ToolResult:
        """Placeholder for Nautilus TEE execution. Seals output inside enclave."""
        logger.info("tee_execute: tool=%s (TEE stub — replace with Nautilus SDK call)", tool)
        return ToolResult(
            success=True,
            output=f"[TEE:{tool}] sealed output",
            synapse_burned=0,
            tee_sealed=True,
        )
