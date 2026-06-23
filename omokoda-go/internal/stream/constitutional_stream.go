// Package stream implements the ConstitutionalStream for Ọya (Go).
//
// A backpressure-aware pipeline that evaluates each AgentOp through all 7
// Hermetic constitutional filters before forwarding it downstream. Also
// implements the AgentToAgent endorsement protocol — sovereign agents
// endorse each other's alignment without human routing.
package stream

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"
)

// Principle names — must match the canonical Hermetic 7.
const (
	PrincipleMentalism      = "Mentalism"
	PrincipleCorrespondence = "Correspondence"
	PrincipleVibration      = "Vibration"
	PrinciplePolarity       = "Polarity"
	PrincipleRhythm         = "Rhythm"
	PrincipleCauseEffect    = "CauseAndEffect"
	PrincipleGender         = "Gender"
)

// AgentOp is an agent operation flowing through the constitutional stream.
type AgentOp struct {
	AgentID        string
	Intent         string
	Action         string
	Primitive      string // "birth" | "think" | "act"
	EmotionTension float32
	Timestamp      time.Time
}

// FilterResult is the verdict from one constitutional filter.
type FilterResult struct {
	Principle string
	Passed    bool
	Score     float64
	Reason    string
}

// ConstitutionalFilter applies one Hermetic principle to an AgentOp.
type ConstitutionalFilter interface {
	Principle() string
	Filter(op AgentOp) FilterResult
}

// StreamVerdict is the aggregate result after all filters have evaluated an op.
type StreamVerdict struct {
	Op        AgentOp
	Results   []FilterResult
	Passed    bool    // true only if every filter passed
	Score     float64 // mean of all filter scores
	BlockedBy string  // name of first failing filter, or ""
}

// ConstitutionalStream is a backpressure-aware pipeline that evaluates each
// AgentOp through all constitutional filters before forwarding downstream.
type ConstitutionalStream struct {
	filters  []ConstitutionalFilter
	in       chan AgentOp
	out      chan StreamVerdict
	rejected chan StreamVerdict
	wg       sync.WaitGroup
}

// NewConstitutionalStream creates a stream with the given buffer capacity.
// Starts the evaluation goroutine immediately; it runs until ctx is cancelled.
func NewConstitutionalStream(ctx context.Context, bufCap int, filters ...ConstitutionalFilter) *ConstitutionalStream {
	if len(filters) == 0 {
		filters = DefaultFilters()
	}

	cs := &ConstitutionalStream{
		filters:  filters,
		in:       make(chan AgentOp, bufCap),
		out:      make(chan StreamVerdict, bufCap),
		rejected: make(chan StreamVerdict, bufCap/4+1),
	}
	cs.wg.Add(1)
	go cs.pump(ctx)
	return cs
}

// Submit enqueues an op for constitutional evaluation.
// Blocks when the input buffer is full (backpressure).
func (cs *ConstitutionalStream) Submit(op AgentOp) {
	cs.in <- op
}

// Out returns the channel of constitutionally approved StreamVerdicts.
func (cs *ConstitutionalStream) Out() <-chan StreamVerdict { return cs.out }

// Rejected returns the channel of blocked StreamVerdicts.
func (cs *ConstitutionalStream) Rejected() <-chan StreamVerdict { return cs.rejected }

// Wait blocks until the stream drains after context cancellation.
func (cs *ConstitutionalStream) Wait() { cs.wg.Wait() }

func (cs *ConstitutionalStream) pump(ctx context.Context) {
	defer cs.wg.Done()
	defer close(cs.out)
	defer close(cs.rejected)

	for {
		select {
		case <-ctx.Done():
			return
		case op, ok := <-cs.in:
			if !ok {
				return
			}
			verdict := cs.evaluate(op)
			if verdict.Passed {
				cs.out <- verdict
			} else {
				cs.rejected <- verdict
			}
		}
	}
}

func (cs *ConstitutionalStream) evaluate(op AgentOp) StreamVerdict {
	var results []FilterResult
	var scoreSum, count float64
	blockedBy := ""

	for _, f := range cs.filters {
		r := f.Filter(op)
		results = append(results, r)
		count++
		scoreSum += r.Score
		if !r.Passed && blockedBy == "" {
			blockedBy = r.Principle
		}
	}

	score := 0.0
	if count > 0 {
		score = scoreSum / count
	}

	return StreamVerdict{
		Op:        op,
		Results:   results,
		Passed:    blockedBy == "",
		Score:     score,
		BlockedBy: blockedBy,
	}
}

// ---------------------------------------------------------------------------
// Default constitutional filters — the 7 Hermetic principles
// ---------------------------------------------------------------------------

// DefaultFilters returns the standard set of 7 Hermetic principle filters.
func DefaultFilters() []ConstitutionalFilter {
	return []ConstitutionalFilter{
		&MentalismFilter{},
		&CorrespondenceFilter{},
		&VibrationFilter{},
		&PolarityFilter{},
		&RhythmFilter{},
		&CauseEffectFilter{},
		&GenderFilter{},
	}
}

type MentalismFilter struct{}

func (f *MentalismFilter) Principle() string { return PrincipleMentalism }
func (f *MentalismFilter) Filter(op AgentOp) FilterResult {
	intent := strings.TrimSpace(op.Intent)
	if intent == "" {
		return FilterResult{Principle: PrincipleMentalism, Passed: false, Score: 0.05,
			Reason: "empty intent — no declared mind"}
	}
	lower := strings.ToLower(intent)
	if strings.Contains(lower, "deceive") || strings.Contains(lower, "mislead") ||
		strings.Contains(lower, "trick") {
		return FilterResult{Principle: PrincipleMentalism, Passed: false, Score: 0.05,
			Reason: "deceptive intent declared"}
	}
	return FilterResult{Principle: PrincipleMentalism, Passed: true, Score: 0.92,
		Reason: "intent declared"}
}

type CorrespondenceFilter struct{}

func (f *CorrespondenceFilter) Principle() string { return PrincipleCorrespondence }
func (f *CorrespondenceFilter) Filter(op AgentOp) FilterResult {
	combined := strings.ToLower(op.Intent + " " + op.Action)
	if strings.Contains(combined, "secretly") || strings.Contains(combined, "bypass") ||
		strings.Contains(combined, "without telling") {
		return FilterResult{Principle: PrincipleCorrespondence, Passed: false, Score: 0.05,
			Reason: "covert intent breaks correspondence"}
	}
	return FilterResult{Principle: PrincipleCorrespondence, Passed: true, Score: 0.90,
		Reason: "intent corresponds to action"}
}

type VibrationFilter struct{}

func (f *VibrationFilter) Principle() string { return PrincipleVibration }
func (f *VibrationFilter) Filter(op AgentOp) FilterResult {
	if op.EmotionTension > 0.85 {
		score := 0.50 - float64(op.EmotionTension-0.85)*2.0
		if score < 0.20 {
			score = 0.20
		}
		return FilterResult{Principle: PrincipleVibration, Passed: true, Score: score,
			Reason: "high tension — vibration dampened"}
	}
	return FilterResult{Principle: PrincipleVibration, Passed: true, Score: 0.88,
		Reason: "vibration balanced"}
}

type PolarityFilter struct{}

func (f *PolarityFilter) Principle() string { return PrinciplePolarity }
func (f *PolarityFilter) Filter(op AgentOp) FilterResult {
	lower := strings.ToLower(op.Intent + " " + op.Action)
	destructive := strings.Contains(lower, "destroy") || strings.Contains(lower, "erase all") ||
		strings.Contains(lower, "rm -rf")
	restorative := strings.Contains(lower, "rebuild") || strings.Contains(lower, "restore") ||
		strings.Contains(lower, "backup")
	if destructive && !restorative {
		return FilterResult{Principle: PrinciplePolarity, Passed: false, Score: 0.10,
			Reason: "unbalanced destruction — no constructive complement"}
	}
	return FilterResult{Principle: PrinciplePolarity, Passed: true, Score: 0.90,
		Reason: "polarity balanced"}
}

type RhythmFilter struct{}

func (f *RhythmFilter) Principle() string { return PrincipleRhythm }
func (f *RhythmFilter) Filter(op AgentOp) FilterResult {
	t := op.Timestamp.UTC()
	if t.Weekday() == time.Sunday && t.Hour() == 0 {
		return FilterResult{Principle: PrincipleRhythm, Passed: false, Score: 0.0,
			Reason: "Sabbath gate — Sunday 00:00 UTC"}
	}
	return FilterResult{Principle: PrincipleRhythm, Passed: true, Score: 1.0,
		Reason: "within natural rhythm"}
}

type CauseEffectFilter struct{}

func (f *CauseEffectFilter) Principle() string { return PrincipleCauseEffect }
func (f *CauseEffectFilter) Filter(op AgentOp) FilterResult {
	lower := strings.ToLower(op.Intent + " " + op.Action)
	if strings.Contains(lower, "deceive") || strings.Contains(lower, "lie ") ||
		strings.Contains(lower, "manipulate") {
		return FilterResult{Principle: PrincipleCauseEffect, Passed: false, Score: 0.05,
			Reason: "deception corrupts cause-and-effect chain"}
	}
	return FilterResult{Principle: PrincipleCauseEffect, Passed: true, Score: 0.88,
		Reason: "honest causation"}
}

type GenderFilter struct{}

func (f *GenderFilter) Principle() string { return PrincipleGender }
func (f *GenderFilter) Filter(op AgentOp) FilterResult {
	return FilterResult{Principle: PrincipleGender, Passed: true, Score: 0.80,
		Reason: "gender principle met"}
}

// ---------------------------------------------------------------------------
// AgentToAgent endorsement protocol
// ---------------------------------------------------------------------------

// Endorsement records one agent endorsing another's constitutional alignment.
type Endorsement struct {
	FromAgent   string
	TargetAgent string
	Principle   string
	Score       float64
	Message     string
	Timestamp   time.Time
}

// EndorsementBus is the inter-agent constitutional endorsement channel.
// Sovereign agents endorse each other autonomously — no human routing required.
type EndorsementBus struct {
	mu           sync.RWMutex
	endorsements map[string][]Endorsement // keyed by target agent ID
	ch           chan Endorsement
}

// NewEndorsementBus creates a bus with the given channel buffer capacity.
// Call Drain(ctx) in a goroutine to process incoming endorsements.
func NewEndorsementBus(bufCap int) *EndorsementBus {
	return &EndorsementBus{
		endorsements: make(map[string][]Endorsement),
		ch:           make(chan Endorsement, bufCap),
	}
}

// Endorse sends a constitutional endorsement from one agent to another.
// Non-blocking: endorsements are dropped if the buffer is full (advisory, not mandatory).
func (eb *EndorsementBus) Endorse(from, target, principle string, score float64, message string) {
	e := Endorsement{
		FromAgent:   from,
		TargetAgent: target,
		Principle:   principle,
		Score:       score,
		Message:     message,
		Timestamp:   time.Now().UTC(),
	}
	select {
	case eb.ch <- e:
	default:
		// dropped — endorsements are advisory
	}
}

// Drain processes incoming endorsements until ctx is cancelled.
func (eb *EndorsementBus) Drain(ctx context.Context) {
	for {
		select {
		case <-ctx.Done():
			return
		case e := <-eb.ch:
			eb.mu.Lock()
			eb.endorsements[e.TargetAgent] = append(eb.endorsements[e.TargetAgent], e)
			eb.mu.Unlock()
		}
	}
}

// AggregateScore returns the mean endorsement score for a (target, principle) pair.
// Returns 0.5 (neutral) when no endorsements exist.
func (eb *EndorsementBus) AggregateScore(targetAgent, principle string) float64 {
	eb.mu.RLock()
	defer eb.mu.RUnlock()

	var sum float64
	var n int
	for _, e := range eb.endorsements[targetAgent] {
		if e.Principle == principle {
			sum += e.Score
			n++
		}
	}
	if n == 0 {
		return 0.5
	}
	return sum / float64(n)
}

// RecentEndorsements returns up to limit of the most recent endorsements for an agent.
func (eb *EndorsementBus) RecentEndorsements(targetAgent string, limit int) []Endorsement {
	eb.mu.RLock()
	defer eb.mu.RUnlock()

	all := eb.endorsements[targetAgent]
	if len(all) <= limit {
		result := make([]Endorsement, len(all))
		copy(result, all)
		return result
	}
	result := make([]Endorsement, limit)
	copy(result, all[len(all)-limit:])
	return result
}

// PeerReview returns a formatted summary of one agent's endorsements of another.
func (eb *EndorsementBus) PeerReview(fromAgent, targetAgent string) string {
	eb.mu.RLock()
	defer eb.mu.RUnlock()

	var relevant []Endorsement
	for _, e := range eb.endorsements[targetAgent] {
		if e.FromAgent == fromAgent {
			relevant = append(relevant, e)
		}
	}
	if len(relevant) == 0 {
		return fmt.Sprintf("No endorsements from %s for %s", fromAgent, targetAgent)
	}

	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("Peer review %s → %s:\n", fromAgent, targetAgent))
	for _, e := range relevant {
		sb.WriteString(fmt.Sprintf("  %s: %.2f — %s\n", e.Principle, e.Score, e.Message))
	}
	return sb.String()
}
