"""
Ògún Memory Tools — the 6 CRUD operations from omo-mem, mapped across the
Omo-Koda2 multi-language stack.

omo-mem pattern mapping:
    memory_list         → list_memory()   — discovers all layers for an agent
    memory_read         → read_memory()   — semantic recall via Ọ̀ṣun (Julia)
    memory_write        → write_memory()  — RACK write with on-chain anchoring
    memory_append       → append_memory() — stream append via Ọya (Go)
    memory_patch        → patch_memory()  — symbolic patch via Ọbàtálá (Lisp)
    memory_delete_lines → delete_lines()  — importance-guarded pruning

Each tool validates its input, routes to the correct Orisha, and returns a
ToolResult.  HTTP clients are thin wrappers — the heavy logic lives in the
language-specific services.
"""

from __future__ import annotations

import hashlib
import time
from dataclasses import dataclass, field
from typing import Optional

import httpx

# Service ports (from ServiceRegistry)
_OSUN_BASE = "http://localhost:4001"   # Julia RACK (memory oracle)
_OYA_BASE  = "http://localhost:4003"   # Go stream manager
_ESHU_BASE = "http://localhost:4000"   # Rust gatekeeper / cache


@dataclass
class ToolResult:
    success: bool
    content: Optional[str] = None
    receipt: Optional[str] = None   # Sui transaction digest if anchored
    error: Optional[str] = None


@dataclass
class MemoryTools:
    """
    The 6 memory CRUD tools.  Pass an httpx.AsyncClient (or the default sync
    client via _sync_client) for unit-testing without running services.
    """

    agent_id: str
    timeout: float = 5.0

    # -------------------------------------------------------------------------
    # memory_list
    # -------------------------------------------------------------------------

    def list_memory(self) -> ToolResult:
        """Enumerate all memory layers and stream dates for this agent."""
        try:
            identity_layer = ["SOUL.md"]

            with httpx.Client(timeout=self.timeout) as c:
                # Long-term from Ọ̀ṣun
                lt_resp = c.get(
                    f"{_OSUN_BASE}/rack/{self.agent_id}/list",
                    params={"layer": "long_term"},
                )
                long_term = lt_resp.json().get("ids", []) if lt_resp.is_success else []

                # Short-term from Ọya
                st_resp = c.get(f"{_OYA_BASE}/memory/{self.agent_id}/list")
                short_term = st_resp.json().get("dates", []) if st_resp.is_success else []

            layers = {
                "identity": identity_layer,
                "long_term": long_term,
                "short_term": short_term,
                "total": 1 + len(long_term) + len(short_term),
            }
            return ToolResult(success=True, content=str(layers))

        except Exception as exc:  # noqa: BLE001
            return ToolResult(success=False, error=str(exc))

    # -------------------------------------------------------------------------
    # memory_read
    # -------------------------------------------------------------------------

    def read_memory(self, query: str, layer: str = "long_term") -> ToolResult:
        """
        Semantic recall via Ọ̀ṣun (Julia RACK).

        For 'short_term', falls back to Ọya keyword search.
        """
        try:
            if layer == "short_term":
                with httpx.Client(timeout=self.timeout) as c:
                    resp = c.get(
                        f"{_OYA_BASE}/memory/{self.agent_id}/search",
                        params={"q": query},
                    )
                content = "\n".join(
                    e["content"] for e in resp.json().get("results", [])
                ) if resp.is_success else ""
            else:
                with httpx.Client(timeout=self.timeout) as c:
                    resp = c.post(
                        f"{_OSUN_BASE}/rack/{self.agent_id}/recall",
                        json={"query": query, "layer": layer, "top_k": 5},
                    )
                content = "\n\n---\n\n".join(
                    e["text"] for e in resp.json().get("cells", [])
                ) if resp.is_success else ""

            return ToolResult(success=True, content=content)

        except Exception as exc:  # noqa: BLE001
            return ToolResult(success=False, error=str(exc))

    # -------------------------------------------------------------------------
    # memory_write
    # -------------------------------------------------------------------------

    def write_memory(
        self,
        content: str,
        layer: str = "long_term",
        importance: int = 3,
    ) -> ToolResult:
        """
        Write a cell to the RACK (Ọ̀ṣun) and anchor on Ṣàngó (Move).

        Identity-layer writes are rejected — use governance flow.
        """
        if layer == "identity":
            return ToolResult(
                success=False,
                error="Identity layer is write-protected; use the governance flow.",
            )

        content_hash = hashlib.sha256(content.encode()).hexdigest()

        try:
            with httpx.Client(timeout=self.timeout) as c:
                resp = c.post(
                    f"{_OSUN_BASE}/rack/{self.agent_id}/write",
                    json={
                        "text": content,
                        "importance": importance / 5.0,
                        "layer": layer,
                        "content_hash": content_hash,
                    },
                )

            if not resp.is_success:
                return ToolResult(success=False, error=resp.text)

            receipt = resp.json().get("receipt")
            return ToolResult(success=True, content=f"Written {len(content)} chars", receipt=receipt)

        except Exception as exc:  # noqa: BLE001
            return ToolResult(success=False, error=str(exc))

    # -------------------------------------------------------------------------
    # memory_append
    # -------------------------------------------------------------------------

    def append_memory(self, content: str, tags: list[str] | None = None) -> ToolResult:
        """
        Append to today's short-term stream via Ọya (Go).

        Does NOT require reading first — append-only by design.
        """
        try:
            with httpx.Client(timeout=self.timeout) as c:
                resp = c.post(
                    f"{_OYA_BASE}/memory/{self.agent_id}/append",
                    json={"content": content, "tags": tags or []},
                )

            if not resp.is_success:
                return ToolResult(success=False, error=resp.text)

            seq = resp.json().get("sequence", 0)
            return ToolResult(success=True, content=f"Appended (seq={seq})")

        except Exception as exc:  # noqa: BLE001
            return ToolResult(success=False, error=str(exc))

    # -------------------------------------------------------------------------
    # memory_patch
    # -------------------------------------------------------------------------

    def patch_memory(self, cell_id: str, find: str, replace: str) -> ToolResult:
        """
        Symbolic find-and-replace via Ọ̀ṣun with Hermetic verification.

        Identity cells require a governance token (not supported in this path).
        """
        try:
            with httpx.Client(timeout=self.timeout) as c:
                resp = c.post(
                    f"{_OSUN_BASE}/rack/{self.agent_id}/patch",
                    json={"cell_id": cell_id, "find": find, "replace": replace},
                )

            if not resp.is_success:
                return ToolResult(success=False, error=resp.text)

            return ToolResult(success=True, content=resp.json().get("result", "patched"))

        except Exception as exc:  # noqa: BLE001
            return ToolResult(success=False, error=str(exc))

    # -------------------------------------------------------------------------
    # memory_delete_lines / prune
    # -------------------------------------------------------------------------

    def delete_lines(
        self,
        cell_id: str,
        from_line: int,
        to_line: int,
    ) -> ToolResult:
        """
        Delete a line range from a RACK cell.

        Refuses if the range contains content marked importance >= 5 (critical).
        """
        if from_line < 1 or to_line < from_line:
            return ToolResult(
                success=False,
                error=f"Invalid range: from_line={from_line}, to_line={to_line}",
            )

        try:
            with httpx.Client(timeout=self.timeout) as c:
                resp = c.post(
                    f"{_OSUN_BASE}/rack/{self.agent_id}/delete_lines",
                    json={"cell_id": cell_id, "from_line": from_line, "to_line": to_line},
                )

            if not resp.is_success:
                return ToolResult(success=False, error=resp.text)

            deleted = resp.json().get("deleted", 0)
            return ToolResult(success=True, content=f"Deleted {deleted} lines")

        except Exception as exc:  # noqa: BLE001
            return ToolResult(success=False, error=str(exc))
