"""
Ọmọ Kọ́dà Python Tool Runner — Ògún / Execution layer.
FastAPI service on :7779.

Architecture:
  - Capability-gated tool registry (Osovm pattern)
  - PII redaction + output sanitisation before every response (Claw-code safety stack)
  - MemCell LRU cache for idempotent tool results (Droidclaw SOMA pattern)
  - Each tool is a pure function: (params: str) → str | raises

Rust's `act` dispatch POSTs to POST /execute for Python-backed tools.
"""

from __future__ import annotations

import hashlib
import json
import logging
import os
import re
import time
from collections import OrderedDict
from typing import Any, Optional

from fastapi import FastAPI, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel

from tools import code_runner, cosmos_generate, data_analysis, web_search

logging.basicConfig(level=logging.INFO, format="%(levelname)s  %(name)s  %(message)s")
log = logging.getLogger("omokoda.tool_runner")

# ---------------------------------------------------------------------------
# Tool registry — name → {fn, required_tier, write, cacheable, description}
# ---------------------------------------------------------------------------

TOOL_REGISTRY: dict[str, dict] = {
    "web_search": {
        "fn": web_search,
        "required_tier": 0,
        "write": False,
        "cacheable": True,
        "description": "Search the web via DuckDuckGo Lite",
    },
    "code_runner": {
        "fn": code_runner,
        "required_tier": 2,
        "write": True,
        "cacheable": False,
        "description": "Execute Python code in an isolated subprocess (Tier 2+)",
    },
    "data_analysis": {
        "fn": data_analysis,
        "required_tier": 1,
        "write": False,
        "cacheable": True,
        "description": "Statistical analysis on JSON/CSV data (Tier 1+)",
    },
    "cosmos": {
        "fn": cosmos_generate,
        "required_tier": 3,
        "write": False,
        "cacheable": False,
        "description": "NVIDIA Cosmos world generation (Tier 3+)",
    },
}

# ---------------------------------------------------------------------------
# MemCell — simple LRU cache keyed by (tool, params) hash (SOMA pattern)
# ---------------------------------------------------------------------------

class MemCell:
    def __init__(self, max_size: int = 256) -> None:
        self._store: OrderedDict[str, str] = OrderedDict()
        self._max = max_size

    def _key(self, tool: str, params: str) -> str:
        return hashlib.blake2b(f"{tool}:{params}".encode(), digest_size=16).hexdigest()

    def get(self, tool: str, params: str) -> Optional[str]:
        k = self._key(tool, params)
        if k in self._store:
            self._store.move_to_end(k)
            return self._store[k]
        return None

    def put(self, tool: str, params: str, value: str) -> None:
        k = self._key(tool, params)
        self._store[k] = value
        self._store.move_to_end(k)
        if len(self._store) > self._max:
            self._store.popitem(last=False)


_cache = MemCell()

# ---------------------------------------------------------------------------
# PII redaction + output sanitisation (Claw-code safety stack)
# ---------------------------------------------------------------------------

_PII_RULES: list[tuple[re.Pattern, str]] = [
    (re.compile(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b"), "[EMAIL]"),
    (re.compile(r"\b\d{3}[-.\s]?\d{3}[-.\s]?\d{4}\b"), "[PHONE]"),
    (re.compile(r"\b\d{3}-\d{2}-\d{4}\b"), "[SSN]"),
    (re.compile(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})\b"), "[CARD]"),
]

_MAX_OUTPUT = 16_384


def redact_pii(text: str) -> str:
    for pattern, replacement in _PII_RULES:
        text = pattern.sub(replacement, text)
    return text


def sanitize_output(text: str) -> str:
    text = "".join(c for c in text if c >= " " or c in "\n\t\r")
    if len(text) > _MAX_OUTPUT:
        text = text[:_MAX_OUTPUT] + "\n[output truncated]"
    return text


def safe_output(raw: str) -> str:
    return sanitize_output(redact_pii(raw))

# ---------------------------------------------------------------------------
# FastAPI app
# ---------------------------------------------------------------------------

app = FastAPI(
    title="Ọmọ Kọ́dà Tool Runner",
    description="Ògún / Execution layer — Python-backed tools for the sovereign agent swarm",
    version="0.1.0",
)


class ExecuteRequest(BaseModel):
    tool: str
    params: str = "{}"
    agent_id: Optional[str] = None
    tier: int = 0


class ExecuteResponse(BaseModel):
    tool: str
    output: str
    cached: bool
    execution_time_ms: int


@app.post("/execute", response_model=ExecuteResponse)
async def execute(req: ExecuteRequest) -> ExecuteResponse:
    entry = TOOL_REGISTRY.get(req.tool)
    if entry is None:
        raise HTTPException(
            status_code=404,
            detail=f"tool '{req.tool}' not found. Available: {list(TOOL_REGISTRY)}",
        )

    required = entry["required_tier"]
    if req.tier < required:
        raise HTTPException(
            status_code=403,
            detail={
                "error": "tier_too_low",
                "required_tier": required,
                "provided_tier": req.tier,
                "tool": req.tool,
            },
        )

    # MemCell lookup for cacheable, read-only tools
    cached = False
    if entry["cacheable"] and not entry["write"]:
        hit = _cache.get(req.tool, req.params)
        if hit is not None:
            log.info("cache hit  tool=%s agent=%s", req.tool, req.agent_id)
            return ExecuteResponse(
                tool=req.tool,
                output=hit,
                cached=True,
                execution_time_ms=0,
            )

    # Execute
    t0 = time.monotonic()
    try:
        raw = entry["fn"](req.params)
    except Exception as exc:
        log.warning("tool error  tool=%s agent=%s: %s", req.tool, req.agent_id, exc)
        raise HTTPException(status_code=500, detail={"error": str(exc), "tool": req.tool})

    elapsed_ms = int((time.monotonic() - t0) * 1000)
    output = safe_output(str(raw))

    if entry["cacheable"] and not entry["write"]:
        _cache.put(req.tool, req.params, output)

    log.info("executed  tool=%s agent=%s tier=%d ms=%d", req.tool, req.agent_id, req.tier, elapsed_ms)
    return ExecuteResponse(tool=req.tool, output=output, cached=cached, execution_time_ms=elapsed_ms)


@app.get("/health")
async def health() -> dict:
    return {"ok": True, "tools": len(TOOL_REGISTRY)}


@app.get("/tools")
async def list_tools() -> dict:
    return {
        name: {
            "description": entry["description"],
            "required_tier": entry["required_tier"],
            "write": entry["write"],
            "cacheable": entry["cacheable"],
        }
        for name, entry in TOOL_REGISTRY.items()
    }


# ---------------------------------------------------------------------------
# /tools/execute — HttpOgunClient contract bridge
# Maps OgunToolReq {tool, input: Value} → execute() → OgunToolResp {output: str}
# ---------------------------------------------------------------------------

class OgunToolRequest(BaseModel):
    tool: str
    input: Any = {}


@app.post("/tools/execute")
async def tools_execute(req: OgunToolRequest) -> dict:
    params_str = json.dumps(req.input) if not isinstance(req.input, str) else req.input
    inner = ExecuteRequest(tool=req.tool, params=params_str, tier=0)
    result = await execute(inner)
    return {"output": result.output}


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import uvicorn

    port = int(os.environ.get("TOOL_RUNNER_PORT", 7779))
    uvicorn.run("server:app", host="0.0.0.0", port=port, reload=False)
