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

// EnforceFlow checks both the Sabbath gate and the agent's rate-limit bucket.
// Returns nil if the action is permitted; otherwise returns a descriptive error.
func (s *Service) EnforceFlow(agentID string, tier uint8) error {
	now := time.Now().UTC()

	// 1. Sabbath gate — time-based rhythm constraint.
	if err := s.gate.Check(now); err != nil {
		return fmt.Errorf("flow denied for agent %s: %w", agentID, err)
	}

	// 2. Rate limiting — token-bucket per agent per tier.
	if err := s.limiter.Allow(agentID, tier); err != nil {
		return fmt.Errorf("flow denied for agent %s: %w", agentID, err)
	}

	return nil
}

// StreamUpdates subscribes agentID to flow updates, writing into ch.
// It sends a heartbeat FlowUpdate every 30 seconds until ch is closed or the
// caller removes the subscription by closing ch externally.
//
// The goroutine terminates when the channel send would block on a closed
// channel, which the caller signals by not reading further and closing ch.
// For cleaner lifecycle management, call UnsubscribeAll when tearing down.
func (s *Service) StreamUpdates(agentID string, ch chan<- FlowUpdate) {
	s.mu.Lock()
	s.subscribers[agentID] = append(s.subscribers[agentID], ch)
	s.mu.Unlock()

	go func() {
		ticker := time.NewTicker(30 * time.Second)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				update := FlowUpdate{
					Event:     "heartbeat",
					Timestamp: time.Now().Unix(),
				}
				// Non-blocking send: if the channel is full or closed, stop.
				select {
				case ch <- update:
				default:
					// Channel is full or closed — stop the goroutine.
					s.removeSubscriber(agentID, ch)
					return
				}
			}
		}
	}()
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
