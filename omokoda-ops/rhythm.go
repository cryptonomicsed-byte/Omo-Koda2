package main

import (
	"sync"
	"time"
)

// RhythmDecision is the outcome of a rhythm gate check.
type RhythmDecision int

const (
	RhythmAllow    RhythmDecision = iota
	RhythmSabbath                 // Irreversible tool blocked on UTC Saturday
	RhythmCooldown                // Per-tool cooldown still active
)

// irreversibleTools mirrors the classification in omokoda-core/src/rhythm.rs.
var irreversibleTools = map[string]bool{
	"write_file":  true,
	"delete_file": true,
	"bash":        true,
	"api_connect": true,
	"edit_file":   true,
	"web_request": true,
}

// toolCooldowns defines the minimum interval between successive calls to a tool.
var toolCooldowns = map[string]time.Duration{
	"bash":        10 * time.Second,
	"web_request": 5 * time.Second,
	"api_connect": 5 * time.Second,
}

type rhythmTracker struct {
	mu       sync.Mutex
	lastUsed map[string]time.Time // key = agentID + ":" + tool
}

var tracker = &rhythmTracker{
	lastUsed: make(map[string]time.Time),
}

// IsSabbath returns true when the current UTC time falls on a Saturday.
func IsSabbath() bool {
	return time.Now().UTC().Weekday() == time.Saturday
}

// CheckRhythm evaluates whether a tool call is permitted at the gateway level.
// This mirrors the enforcement in the Rust Steward but runs first at the Go
// boundary so callers get an early rejection without spending a round-trip.
func CheckRhythm(tool, agentID string) RhythmDecision {
	if IsSabbath() && irreversibleTools[tool] {
		return RhythmSabbath
	}

	if cd, ok := toolCooldowns[tool]; ok {
		key := agentID + ":" + tool
		tracker.mu.Lock()
		last, seen := tracker.lastUsed[key]
		if seen && time.Since(last) < cd {
			tracker.mu.Unlock()
			return RhythmCooldown
		}
		tracker.lastUsed[key] = time.Now()
		tracker.mu.Unlock()
	}

	return RhythmAllow
}
