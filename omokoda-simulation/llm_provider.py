"""LLM provider with SOMA context injection and IRIS routing."""

from __future__ import annotations

import logging
import os
from dataclasses import dataclass, field
from typing import Optional

import aiohttp

log = logging.getLogger("omokoda.llm_provider")

_TIMEOUT = aiohttp.ClientTimeout(total=30)

# ---------------------------------------------------------------------------
# IRIS routing
# ---------------------------------------------------------------------------

@dataclass
class IrisRouting:
    profile: str          # "reflex" | "balanced" | "sharp" | "deep" | "gentle"
    max_tokens: int
    temperature: float
    style_guidance: str
    warmth_boost: bool

    @classmethod
    def from_profile(cls, profile: str) -> "IrisRouting":
        """Map profile names to parameters (match Rust iris.rs values)."""
        profiles: dict[str, IrisRouting] = {
            "reflex":   cls("reflex",   256,  0.3, "Be extremely concise. One sentence max.", False),
            "balanced": cls("balanced", 1024, 0.7, "Balance clarity and depth.", False),
            "sharp":    cls("sharp",    2048, 0.2, "Maximum technical precision. Show your reasoning.", False),
            "deep":     cls("deep",     4096, 0.8, "Think deeply. Explore all angles before concluding.", False),
            "gentle":   cls("gentle",   1024, 0.9, "Lead with empathy. Be warm, patient, and supportive.", True),
        }
        return profiles.get(profile, profiles["balanced"])


# ---------------------------------------------------------------------------
# SOMA context
# ---------------------------------------------------------------------------

@dataclass
class SomaContext:
    system_prompt: str
    emotion_description: str = ""
    lpm_context: str = ""
    recent_memories: list[str] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Message
# ---------------------------------------------------------------------------

@dataclass
class Message:
    role: str   # "user" | "assistant" | "system"
    content: str


# ---------------------------------------------------------------------------
# LLM provider
# ---------------------------------------------------------------------------

class LlmProvider:
    """Unified provider — Anthropic or OpenAI, auto-detected from base_url."""

    _ANTHROPIC_HOST = "api.anthropic.com"
    _ANTHROPIC_VERSION = "2023-06-01"
    _HISTORY_LIMIT = 20

    def __init__(self, api_key: str, base_url: str, model: str) -> None:
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.model = model
        self.is_anthropic = self._ANTHROPIC_HOST in self.base_url

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def chat(
        self,
        soma: SomaContext,
        user_message: str,
        routing: IrisRouting,
        history: list[Message],
    ) -> str:
        """Full chat with SOMA + IRIS context injected into system prompt."""
        system = self._build_system_prompt(soma, routing)

        # Trim history to last _HISTORY_LIMIT messages then append current turn
        trimmed = history[-self._HISTORY_LIMIT:]
        messages = [{"role": m.role, "content": m.content} for m in trimmed]
        messages.append({"role": "user", "content": user_message})

        if self.is_anthropic:
            return await self._anthropic(system, messages, routing)
        return await self._openai(system, messages, routing)

    async def raw_chat(self, prompt: str) -> str:
        """Minimal single-turn chat for internal operations (learning, reflection, SOMA consolidation)."""
        routing = IrisRouting.from_profile("balanced")
        messages = [{"role": "user", "content": prompt}]
        if self.is_anthropic:
            return await self._anthropic("", messages, routing)
        return await self._openai(None, messages, routing)

    # ------------------------------------------------------------------
    # Backend implementations
    # ------------------------------------------------------------------

    async def _anthropic(
        self,
        system: str,
        messages: list[dict],
        routing: IrisRouting,
    ) -> str:
        """POST to Anthropic Messages API."""
        url = f"{self.base_url}/v1/messages"
        headers = {
            "x-api-key": self.api_key,
            "anthropic-version": self._ANTHROPIC_VERSION,
            "content-type": "application/json",
        }
        payload: dict = {
            "model": self.model,
            "max_tokens": routing.max_tokens,
            "temperature": routing.temperature,
            "messages": messages,
        }
        if system:
            payload["system"] = system

        log.debug("anthropic request  model=%s profile=%s", self.model, routing.profile)
        async with aiohttp.ClientSession(timeout=_TIMEOUT) as session:
            async with session.post(url, headers=headers, json=payload) as resp:
                if resp.status >= 300:
                    body = await resp.text()
                    raise RuntimeError(
                        f"Anthropic API error {resp.status}: {body}"
                    )
                data = await resp.json()

        # Anthropic response shape: {"content": [{"type": "text", "text": "..."}], ...}
        text: str = data["content"][0]["text"]
        return text.strip()

    async def _openai(
        self,
        system: Optional[str],
        messages: list[dict],
        routing: IrisRouting,
    ) -> str:
        """POST to OpenAI-compatible chat completions."""
        url = f"{self.base_url}/v1/chat/completions"
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "content-type": "application/json",
        }

        # Prepend system message when provided
        full_messages: list[dict] = []
        if system:
            full_messages.append({"role": "system", "content": system})
        full_messages.extend(messages)

        payload = {
            "model": self.model,
            "max_tokens": routing.max_tokens,
            "temperature": routing.temperature,
            "messages": full_messages,
        }

        log.debug("openai request  model=%s profile=%s", self.model, routing.profile)
        async with aiohttp.ClientSession(timeout=_TIMEOUT) as session:
            async with session.post(url, headers=headers, json=payload) as resp:
                if resp.status >= 300:
                    body = await resp.text()
                    raise RuntimeError(
                        f"OpenAI API error {resp.status}: {body}"
                    )
                data = await resp.json()

        # OpenAI response shape: {"choices": [{"message": {"content": "..."}}], ...}
        text: str = data["choices"][0]["message"]["content"]
        return text.strip()

    # ------------------------------------------------------------------
    # Prompt assembly
    # ------------------------------------------------------------------

    def _build_system_prompt(self, soma: SomaContext, routing: IrisRouting) -> str:
        """Assemble: base prompt + IRIS routing directive + emotion state + LPM context."""
        parts: list[str] = []

        # 1. Base system prompt from SOMA
        if soma.system_prompt:
            parts.append(soma.system_prompt.strip())

        # 2. IRIS routing directive
        iris_section = f"[IRIS:{routing.profile.upper()}] {routing.style_guidance}"
        if routing.warmth_boost:
            iris_section += "\nWarmth is your highest priority right now."
        parts.append(iris_section)

        # 3. Emotional state
        if soma.emotion_description:
            parts.append(f"[SOMA:EMOTION] {soma.emotion_description.strip()}")

        # 4. LPM context (long-term personality / memory context)
        if soma.lpm_context:
            parts.append(f"[SOMA:LPM] {soma.lpm_context.strip()}")

        # 5. Recent memories (episodic)
        if soma.recent_memories:
            memory_lines = "\n".join(f"- {m}" for m in soma.recent_memories)
            parts.append(f"[SOMA:MEMORIES]\n{memory_lines}")

        return "\n\n".join(parts)


# ---------------------------------------------------------------------------
# Convenience factory from environment variables
# ---------------------------------------------------------------------------

def provider_from_env() -> LlmProvider:
    """Create an LlmProvider from standard environment variables.

    Required env vars:
        LLM_API_KEY   — API key
        LLM_BASE_URL  — Base URL (e.g. https://api.anthropic.com or https://api.openai.com)
        LLM_MODEL     — Model name (e.g. claude-opus-4-5 or gpt-4o)
    """
    api_key = os.environ["LLM_API_KEY"]
    base_url = os.environ["LLM_BASE_URL"]
    model = os.environ["LLM_MODEL"]
    return LlmProvider(api_key=api_key, base_url=base_url, model=model)
