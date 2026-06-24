// Package flow implements the ỌYA FlowService.
// It combines rate limiting and Sabbath gate enforcement and provides
// a fan-out streaming channel for real-time flow updates.
package flow

import (
	"fmt"
	"sync"
	"time"

	"github.com/omo-koda/omokoda-go/internal/ratelimit"
	"github.com/omo-koda/omokoda-go/internal/sabbath"
)

// FlowUpdate is a single event pushed to subscribed agents.
type FlowUpdate struct {
	Event     string
	Timestamp int64 // Unix timestamp (seconds)
}

// Service is the ỌYA FlowService implementation.
type Service struct {
	limiter *ratelimit.Limiter
	gate    *sabbath.Gate

	mu          sync.RWMutex
	subscribers map[string][]chan<- FlowUpdate // agentID → open channels
}

// New constructs a FlowService with default rate-limit TTL (1 hour).
func New() *Service {
	return &Service{
		limiter:     ratelimit.New(time.Hour),
		gate:        sabbath.New(),
		subscribers: make(map[string][]chan<- FlowUpdate),
	}
}

// EnforceFlow checks rate limit and Sabbath gate for an agent+tier combination.
// Returns nil on allow, or an error message on deny.
func (s *FlowService) EnforceFlow(agentID string, tier int) error {
	if isSabbath() {
		return fmt.Errorf("rhythm_constraint: Saturday 00:00-01:00 UTC — Sabbath gate active, no actions allowed")
	}

	// 2. Rate limiting — token-bucket per agent per tier.
	if err := s.limiter.Allow(agentID, tier); err != nil {
		return fmt.Errorf("flow denied for agent %s: %w", agentID, err)
	}

	return nil
}

// isSabbath returns true during UTC Saturday 00:00–01:00 (ritual-codex Sabbath enforcement).
func isSabbath() bool {
	now := time.Now().UTC()
	return now.Weekday() == time.Saturday && now.Hour() == 0
}

// BroadcastUpdate fans out update to every channel subscribed across all agents.
// Dead (full) channels are removed on the next delivery attempt.
func (s *Service) BroadcastUpdate(update FlowUpdate) {
	s.mu.RLock()
	// Snapshot the subscriber map to avoid holding the lock during sends.
	snapshot := make(map[string][]chan<- FlowUpdate, len(s.subscribers))
	for id, chans := range s.subscribers {
		snapshot[id] = append([]chan<- FlowUpdate(nil), chans...)
	}
	s.mu.RUnlock()

	for agentID, chans := range snapshot {
		for _, ch := range chans {
			select {
			case ch <- update:
			default:
				// Channel is stalled or closed — evict it.
				s.removeSubscriber(agentID, ch)
			}
		}
	}
}

// removeSubscriber removes a single channel from an agent's subscription list.
func (s *Service) removeSubscriber(agentID string, target chan<- FlowUpdate) {
	s.mu.Lock()
	defer s.mu.Unlock()

	chans := s.subscribers[agentID]
	filtered := chans[:0]
	for _, ch := range chans {
		if ch != target {
			filtered = append(filtered, ch)
		}
	}
	if len(filtered) == 0 {
		delete(s.subscribers, agentID)
	} else {
		s.subscribers[agentID] = filtered
	}
}

// UnsubscribeAll removes all subscriptions for agentID.
// Callers should close any channels they own after calling this.
func (s *Service) UnsubscribeAll(agentID string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.subscribers, agentID)
}

// Stop shuts down the underlying rate limiter's cleanup goroutine.
func (s *Service) Stop() {
	s.limiter.Stop()
}
