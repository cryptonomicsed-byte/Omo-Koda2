// Package ratelimit implements a per-agent, per-tier token bucket rate limiter.
//
// Tiers and their limits:
//
//	T0:  1 req/s, burst  5
//	T1:  2 req/s, burst 10
//	T2:  5 req/s, burst 20
//	T3: 10 req/s, burst 40
//	T4: 20 req/s, burst 80
//	T5: 50 req/s, burst 200
//
// Buckets are keyed by agentID and expire after 1 hour of inactivity.
package ratelimit

import (
	"errors"
	"fmt"
	"sync"
	"time"
)

// ErrRateLimited is returned when an agent has exhausted its token bucket.
var ErrRateLimited = errors.New("rate_limited")

// tierConfig holds the rate (tokens/sec) and burst size for one tier.
type tierConfig struct {
	rate  float64 // tokens added per second
	burst float64 // maximum token capacity
}

// tierConfigs maps tier index (0–5) to its configuration.
var tierConfigs = [6]tierConfig{
	{rate: 1, burst: 5},   // T0
	{rate: 2, burst: 10},  // T1
	{rate: 5, burst: 20},  // T2
	{rate: 10, burst: 40}, // T3
	{rate: 20, burst: 80}, // T4
	{rate: 50, burst: 200}, // T5
}

// bucket is a single token-bucket entry stored in the limiter map.
type bucket struct {
	mu       sync.Mutex
	tokens   float64
	lastSeen time.Time
	cfg      tierConfig
}

// allow attempts to consume one token.
// It refills the bucket based on elapsed time since the last call.
// Returns true if a token was available.
func (b *bucket) allow(now time.Time) bool {
	b.mu.Lock()
	defer b.mu.Unlock()

	elapsed := now.Sub(b.lastSeen).Seconds()
	if elapsed > 0 {
		b.tokens += elapsed * b.cfg.rate
		if b.tokens > b.cfg.burst {
			b.tokens = b.cfg.burst
		}
		b.lastSeen = now
	}

	if b.tokens < 1 {
		return false
	}
	b.tokens--
	return true
}

// bucketEntry wraps a bucket with an expiry timestamp tracked externally.
type bucketEntry struct {
	b          *bucket
	lastAccess time.Time
	mu         sync.Mutex
}

// Limiter manages per-agent token buckets.
type Limiter struct {
	buckets sync.Map // map[string]*bucketEntry
	ttl     time.Duration

	// cleanupTicker drives periodic expiry of idle buckets.
	stopCleanup chan struct{}
}

// New creates a Limiter that expires idle buckets after ttl.
// A zero ttl defaults to 1 hour.
func New(ttl time.Duration) *Limiter {
	if ttl <= 0 {
		ttl = time.Hour
	}
	l := &Limiter{
		ttl:         ttl,
		stopCleanup: make(chan struct{}),
	}
	go l.cleanup()
	return l
}

// cleanup runs every ttl/2, removing buckets that have been idle for >= ttl.
func (l *Limiter) cleanup() {
	interval := l.ttl / 2
	if interval < time.Second {
		interval = time.Second
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ticker.C:
			now := time.Now()
			l.buckets.Range(func(key, val any) bool {
				entry := val.(*bucketEntry)
				entry.mu.Lock()
				idle := now.Sub(entry.lastAccess)
				entry.mu.Unlock()
				if idle >= l.ttl {
					l.buckets.Delete(key)
				}
				return true
			})
		case <-l.stopCleanup:
			return
		}
	}
}

// Stop shuts down the background cleanup goroutine.
func (l *Limiter) Stop() {
	close(l.stopCleanup)
}

// Allow checks whether agentID with the given tier may proceed.
// tier must be in [0, 5]; out-of-range values are clamped to T0.
// Returns nil if the request is permitted, ErrRateLimited otherwise.
func (l *Limiter) Allow(agentID string, tier uint8) error {
	if tier > 5 {
		tier = 0
	}
	cfg := tierConfigs[tier]
	now := time.Now()

	raw, _ := l.buckets.LoadOrStore(agentID, &bucketEntry{
		b: &bucket{
			tokens:   cfg.burst, // start full
			lastSeen: now,
			cfg:      cfg,
		},
		lastAccess: now,
	})
	entry := raw.(*bucketEntry)

	// Update last-access time for TTL tracking.
	entry.mu.Lock()
	entry.lastAccess = now
	entry.mu.Unlock()

	// Ensure the bucket uses the current tier config (in case tier changed).
	entry.b.mu.Lock()
	entry.b.cfg = cfg
	entry.b.mu.Unlock()

	if !entry.b.allow(now) {
		return fmt.Errorf("%w: agent %s tier T%d", ErrRateLimited, agentID, tier)
	}
	return nil
}
