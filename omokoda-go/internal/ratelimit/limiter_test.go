package ratelimit

import (
	"testing"
)

func TestLimiter_AllowsUpToBurst(t *testing.T) {
	l := New()
	// Tier 1: burst=10
	allowed := 0
	for i := 0; i < 15; i++ {
		if l.Allow("agent-test", 1) {
			allowed++
		}
	}
	if allowed < 10 {
		t.Errorf("expected at least 10 allowed within burst, got %d", allowed)
	}
	if allowed == 15 {
		t.Error("expected some requests to be denied after burst exhausted")
	}
}

func TestLimiter_UnknownTierFallsBackToTier0(t *testing.T) {
	l := New()
	// Tier 99 doesn't exist, should fallback to tier 0 (burst=5)
	allowed := 0
	for i := 0; i < 10; i++ {
		if l.Allow("agent-x", 99) {
			allowed++
		}
	}
	if allowed == 0 {
		t.Error("expected at least some requests allowed")
	}
}
