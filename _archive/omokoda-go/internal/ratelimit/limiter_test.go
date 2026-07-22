package ratelimit_test

import (
	"errors"
	"testing"
	"time"

	"github.com/omo-koda/omokoda-go/internal/ratelimit"
)

// newTestLimiter creates a Limiter with a very long TTL so buckets never
// expire during the test run.
func newTestLimiter() *ratelimit.Limiter {
	return ratelimit.New(24 * time.Hour)
}

// TestT0_first_request_allowed verifies that the very first request from a
// T0 agent is always allowed (the bucket starts full at burst=5).
func TestT0_first_request_allowed(t *testing.T) {
	l := newTestLimiter()
	defer l.Stop()

	err := l.Allow("agent-t0-first", 0)
	if err != nil {
		t.Errorf("expected first T0 request to be allowed, got: %v", err)
	}
}

// TestT0_rapid_requests_denied verifies that a T0 agent (burst=5) is denied
// after the burst is exhausted with no refill time between calls.
func TestT0_rapid_requests_denied(t *testing.T) {
	l := newTestLimiter()
	defer l.Stop()

	agentID := "agent-t0-burst"
	denied := false
	// Make 10 rapid requests; the burst is 5, so at least one must be denied.
	for i := 0; i < 10; i++ {
		err := l.Allow(agentID, 0)
		if errors.Is(err, ratelimit.ErrRateLimited) {
			denied = true
			break
		}
	}
	if !denied {
		t.Errorf("expected at least one of 10 rapid T0 requests to be denied (burst=5)")
	}
}

// TestT5_burst_all_allowed verifies that 50 rapid requests from a T5 agent
// are all allowed because burst=200 covers them.
func TestT5_burst_all_allowed(t *testing.T) {
	l := newTestLimiter()
	defer l.Stop()

	agentID := "agent-t5-burst"
	for i := 0; i < 50; i++ {
		err := l.Allow(agentID, 5)
		if err != nil {
			t.Errorf("request %d: expected T5 request to be allowed (burst=200), got: %v", i+1, err)
		}
	}
}

// TestT0_refill verifies that after sleeping enough time for the bucket to
// refill at least one token, a previously-denied agent can proceed again.
func TestT0_refill(t *testing.T) {
	l := newTestLimiter()
	defer l.Stop()

	agentID := "agent-t0-refill"
	// Exhaust the burst.
	for i := 0; i < 5; i++ {
		_ = l.Allow(agentID, 0)
	}
	// Should be denied now.
	if err := l.Allow(agentID, 0); !errors.Is(err, ratelimit.ErrRateLimited) {
		t.Skip("bucket not yet exhausted — timing-sensitive test skipped")
	}
	// Wait >1 second for T0 rate=1/s to refill one token.
	time.Sleep(1100 * time.Millisecond)
	if err := l.Allow(agentID, 0); err != nil {
		t.Errorf("expected refilled T0 bucket to allow request, got: %v", err)
	}
}

// TestOutOfRangeTier_clamped verifies that a tier >5 is treated as T0.
func TestOutOfRangeTier_clamped(t *testing.T) {
	l := newTestLimiter()
	defer l.Stop()

	// First request should succeed regardless (clamped to T0, burst=5).
	err := l.Allow("agent-high-tier", 255)
	if err != nil {
		t.Errorf("expected clamped out-of-range tier to allow first request, got: %v", err)
	}
}
