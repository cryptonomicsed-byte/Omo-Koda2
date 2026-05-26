from __future__ import annotations
import enum
import logging
from typing import Any, Optional

logger = logging.getLogger(__name__)


class PrivacyMode(enum.Enum):
    PUBLIC = "public"
    PRIVATE = "private"
    INCOGNITO = "incognito"


_LOCAL_PROVIDERS = {"webllm", "ollama", "local"}


class PrivacyViolationError(RuntimeError):
    """Raised when a private/incognito agent is routed to an external provider."""


class OgunExecutor:
    """Ògún tool executor with privacy routing, tier gating, and synapse cost tracking."""

    def __init__(self, agent_id: str, tier: int = 0, privacy_mode: PrivacyMode = PrivacyMode.PUBLIC):
        self.agent_id = agent_id
        self.tier = tier
        self.privacy_mode = privacy_mode

    def execute_tool(self, tool_name: str, params: dict[str, Any], provider: str = "webllm") -> dict[str, Any]:
        """Execute a tool after tier and privacy checks."""
        if not self._tier_allowed(tool_name):
            return {"error": f"tool '{tool_name}' not allowed at tier {self.tier}"}

        self._check_privacy_routing(provider)

        synapse_cost = self._compute_synapse_cost(tool_name)
        logger.info(
            "[OgunExecutor] agent=%s tool=%s tier=%d provider=%s cost=%d",
            self.agent_id, tool_name, self.tier, provider, synapse_cost,
        )

        if self.privacy_mode == PrivacyMode.INCOGNITO:
            return self._execute_no_log(tool_name, params, synapse_cost)
        return self._execute(tool_name, params, synapse_cost)

    def _check_privacy_routing(self, provider: str) -> None:
        """Hard-fail if private/incognito is routed to an external provider."""
        if self.privacy_mode in (PrivacyMode.PRIVATE, PrivacyMode.INCOGNITO):
            if provider.lower() not in _LOCAL_PROVIDERS:
                raise PrivacyViolationError(
                    f"/private mode requires a local provider (webllm/ollama); "
                    f"'{provider}' is external. Hard fail — no silent reroute."
                )

    def _tier_allowed(self, tool_name: str) -> bool:
        tier_tools = {
            0: {"web_search", "note_taking", "read_file", "glob", "grep"},
            1: {"web_search", "note_taking", "read_file", "glob", "grep", "image_gen_basic"},
            2: {"web_search", "note_taking", "read_file", "glob", "grep", "image_gen_basic", "code_runner", "bash"},
        }
        allowed = set()
        for t in range(min(self.tier + 1, 3)):
            allowed |= tier_tools.get(t, set())
        if self.tier >= 3:
            allowed |= {"data_analysis", "api_connect"}
        if self.tier >= 4:
            allowed |= {"agent_orchestration"}
        if self.tier >= 5:
            allowed |= {"self_modification", "multi_agent_fabric"}
        return tool_name in allowed

    def _compute_synapse_cost(self, tool_name: str) -> int:
        base_costs = {
            "web_search": 10,
            "bash": 50,
            "code_runner": 30,
            "agent_orchestration": 200,
            "self_modification": 500,
        }
        return base_costs.get(tool_name, 5)

    def _execute(self, tool_name: str, params: dict, synapse_cost: int) -> dict:
        return {"tool": tool_name, "params": params, "synapse_cost": synapse_cost, "status": "ok"}

    def _execute_no_log(self, tool_name: str, params: dict, synapse_cost: int) -> dict:
        # INCOGNITO: execute without logging
        return {"tool": tool_name, "synapse_cost": synapse_cost, "status": "ok"}

    def _tee_execute_stub(self, tool_name: str, params: dict) -> dict:
        """Placeholder for TEE integration via nautilus_integration."""
        return {"tool": tool_name, "tee": True, "status": "stub"}
