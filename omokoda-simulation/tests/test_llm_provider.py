"""Unit tests for llm_provider — prompt-building logic only, no HTTP mocking."""

from __future__ import annotations

import sys
import os
import unittest

# Ensure the simulation package root is on the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from llm_provider import IrisRouting, LlmProvider, SomaContext


# ---------------------------------------------------------------------------
# IrisRouting
# ---------------------------------------------------------------------------

class TestIrisRouting(unittest.TestCase):

    def test_from_profile_balanced(self):
        r = IrisRouting.from_profile("balanced")
        self.assertEqual(r.profile, "balanced")
        self.assertEqual(r.max_tokens, 1024)
        self.assertAlmostEqual(r.temperature, 0.7)
        self.assertFalse(r.warmth_boost)

    def test_from_profile_unknown_falls_back_to_balanced(self):
        r = IrisRouting.from_profile("nonexistent_profile")
        self.assertEqual(r.profile, "balanced")
        self.assertEqual(r.max_tokens, 1024)
        self.assertAlmostEqual(r.temperature, 0.7)

    def test_all_profiles_have_guidance(self):
        for name in ("reflex", "balanced", "sharp", "deep", "gentle"):
            with self.subTest(profile=name):
                r = IrisRouting.from_profile(name)
                self.assertIsInstance(r.style_guidance, str)
                self.assertTrue(len(r.style_guidance) > 0, f"style_guidance empty for profile '{name}'")

    def test_reflex_has_short_tokens(self):
        r = IrisRouting.from_profile("reflex")
        self.assertEqual(r.max_tokens, 256)
        self.assertAlmostEqual(r.temperature, 0.3)

    def test_deep_has_large_tokens(self):
        r = IrisRouting.from_profile("deep")
        self.assertEqual(r.max_tokens, 4096)

    def test_gentle_has_warmth_boost(self):
        r = IrisRouting.from_profile("gentle")
        self.assertTrue(r.warmth_boost)

    def test_sharp_no_warmth_boost(self):
        r = IrisRouting.from_profile("sharp")
        self.assertFalse(r.warmth_boost)


# ---------------------------------------------------------------------------
# SomaContext
# ---------------------------------------------------------------------------

class TestSomaContext(unittest.TestCase):

    def test_defaults(self):
        soma = SomaContext(system_prompt="You are a helpful agent.")
        self.assertEqual(soma.emotion_description, "")
        self.assertEqual(soma.lpm_context, "")
        self.assertIsInstance(soma.recent_memories, list)
        self.assertEqual(len(soma.recent_memories), 0)

    def test_recent_memories_not_shared_across_instances(self):
        """Mutable default must not be shared between dataclass instances."""
        a = SomaContext(system_prompt="A")
        b = SomaContext(system_prompt="B")
        a.recent_memories.append("memory")
        self.assertEqual(len(b.recent_memories), 0)


# ---------------------------------------------------------------------------
# LlmProvider — _build_system_prompt (no HTTP)
# ---------------------------------------------------------------------------

def _make_provider(base_url: str = "https://api.openai.com") -> LlmProvider:
    return LlmProvider(api_key="test-key", base_url=base_url, model="test-model")


class TestLlmProvider(unittest.TestCase):

    def test_is_anthropic_detected(self):
        p = _make_provider("https://api.anthropic.com")
        self.assertTrue(p.is_anthropic)

    def test_is_openai_detected(self):
        p = _make_provider("https://api.openai.com")
        self.assertFalse(p.is_anthropic)

    def test_build_system_prompt_includes_iris_guidance(self):
        provider = _make_provider()
        soma = SomaContext(system_prompt="Base system prompt.")
        routing = IrisRouting.from_profile("sharp")
        result = provider._build_system_prompt(soma, routing)
        self.assertIn(routing.style_guidance, result)
        self.assertIn("[IRIS:SHARP]", result)

    def test_build_system_prompt_includes_emotion(self):
        provider = _make_provider()
        soma = SomaContext(
            system_prompt="Base.",
            emotion_description="Curious and alert",
        )
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        self.assertIn("Curious and alert", result)
        self.assertIn("[SOMA:EMOTION]", result)

    def test_build_system_prompt_includes_warmth_when_gentle(self):
        provider = _make_provider()
        soma = SomaContext(system_prompt="Base.")
        routing = IrisRouting.from_profile("gentle")
        result = provider._build_system_prompt(soma, routing)
        self.assertIn("Warmth is your highest priority right now.", result)

    def test_build_system_prompt_no_emotion_section_when_empty(self):
        provider = _make_provider()
        soma = SomaContext(system_prompt="Base.")
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        self.assertNotIn("[SOMA:EMOTION]", result)

    def test_build_system_prompt_includes_lpm_context(self):
        provider = _make_provider()
        soma = SomaContext(system_prompt="Base.", lpm_context="Persistent trait: directness.")
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        self.assertIn("[SOMA:LPM]", result)
        self.assertIn("Persistent trait: directness.", result)

    def test_build_system_prompt_includes_memories(self):
        provider = _make_provider()
        soma = SomaContext(
            system_prompt="Base.",
            recent_memories=["User prefers brevity", "Last topic: Rust lifetimes"],
        )
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        self.assertIn("[SOMA:MEMORIES]", result)
        self.assertIn("User prefers brevity", result)
        self.assertIn("Last topic: Rust lifetimes", result)

    def test_build_system_prompt_no_memories_section_when_empty(self):
        provider = _make_provider()
        soma = SomaContext(system_prompt="Base.")
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        self.assertNotIn("[SOMA:MEMORIES]", result)

    def test_build_system_prompt_includes_base_prompt(self):
        provider = _make_provider()
        soma = SomaContext(system_prompt="You are Ọmọ Kọ́dà.")
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        self.assertIn("You are Ọmọ Kọ́dà.", result)

    def test_build_system_prompt_sections_ordered(self):
        """Base prompt appears before IRIS, IRIS before EMOTION."""
        provider = _make_provider()
        soma = SomaContext(
            system_prompt="Base.",
            emotion_description="Focused",
        )
        routing = IrisRouting.from_profile("balanced")
        result = provider._build_system_prompt(soma, routing)
        base_pos = result.index("Base.")
        iris_pos = result.index("[IRIS:")
        emotion_pos = result.index("[SOMA:EMOTION]")
        self.assertLess(base_pos, iris_pos)
        self.assertLess(iris_pos, emotion_pos)

    def test_base_url_trailing_slash_stripped(self):
        p = LlmProvider(api_key="k", base_url="https://api.openai.com/", model="m")
        self.assertFalse(p.base_url.endswith("/"))


if __name__ == "__main__":
    unittest.main()
