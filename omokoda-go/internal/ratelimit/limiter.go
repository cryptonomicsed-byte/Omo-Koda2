package ratelimit

import (
	"sync"
	"time"
)

// Tier-based token bucket: (req/sec, burst) per tier.
var tierLimits = map[int][2]float64{
	0: {1, 5},
	1: {2, 10},
	2: {5, 20},
	3: {10, 40},
	4: {20, 80},
	5: {50, 200},
}

type bucket struct {
	tokens   float64
	maxBurst float64
	rate     float64 // tokens per second
	lastSeen time.Time
}

func (b *bucket) allow() bool {
	now := time.Now()
	elapsed := now.Sub(b.lastSeen).Seconds()
	b.tokens = min64(b.maxBurst, b.tokens+elapsed*b.rate)
	b.lastSeen = now
	if b.tokens >= 1.0 {
		b.tokens--
		return true
	}
	return false
}

func min64(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

// Limiter is a per-agent, tier-aware token bucket rate limiter.
type Limiter struct {
	mu      sync.Mutex
	buckets map[string]*bucket
}

func New() *Limiter {
	return &Limiter{buckets: make(map[string]*bucket)}
}

// Allow returns true if the agent+tier combination is within rate limits.
func (l *Limiter) Allow(agentID string, tier int) bool {
	limits, ok := tierLimits[tier]
	if !ok {
		limits = tierLimits[0]
	}
	key := agentID + ":" + string(rune('0'+tier))

	l.mu.Lock()
	defer l.mu.Unlock()

	b, exists := l.buckets[key]
	if !exists {
		b = &bucket{
			tokens:   limits[1],
			maxBurst: limits[1],
			rate:     limits[0],
			lastSeen: time.Now(),
		}
		l.buckets[key] = b
	}
	return b.allow()
}
