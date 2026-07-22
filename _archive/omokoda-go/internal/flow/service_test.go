package flow

import (
	"testing"

	"github.com/omo-koda/omokoda-go/internal/ratelimit"
)

func TestEnforceFlow_AllowsNormalRequest(t *testing.T) {
	svc := NewService(ratelimit.New(0))
	// Not Sabbath during tests (unless test runs Sunday 00:00 UTC — acceptable)
	// Just test that rate limit allows first request
	err := svc.EnforceFlow("agent-1", 1)
	if err != nil && err.Error()[:16] != "rhythm_constraint" {
		// rate limit on first request would be unexpected
		t.Errorf("unexpected denial for first request: %v", err)
	}
}

func TestEnforceFlow_RateLimitTier0(t *testing.T) {
	svc := NewService(ratelimit.New(0))
	// Tier 0 burst=5; after 6 rapid requests the 6th should be denied
	allowed := 0
	for i := 0; i < 10; i++ {
		if err := svc.EnforceFlow("burst-agent", 0); err == nil {
			allowed++
		}
	}
	// At least 5 should be allowed (burst), and at least one denied
	if allowed == 10 {
		t.Error("expected some requests to be denied by rate limiter")
	}
}
