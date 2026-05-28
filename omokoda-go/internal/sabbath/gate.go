// Package sabbath enforces the Sabbath rhythm gate for ỌYA.
// UTC Sunday 00:00–01:00 is restricted — no agent actions are permitted.
package sabbath

import (
	"errors"
	"fmt"
	"time"
)

// ErrRhythmConstraint is returned when an action is blocked by the Sabbath gate.
var ErrRhythmConstraint = errors.New("rhythm_constraint")

// Gate enforces Sabbath timing rules.
type Gate struct{}

// New returns a new Gate.
func New() *Gate { return &Gate{} }

// IsRestricted reports whether t falls within the Sabbath window
// (Sunday 00:00:00 UTC inclusive to 01:00:00 UTC exclusive).
func (g *Gate) IsRestricted(t time.Time) bool {
	u := t.UTC()
	if u.Weekday() != time.Sunday {
		return false
	}
	// Restricted during the first hour only: 00:00:00 <= t < 01:00:00
	return u.Hour() == 0
}

// NextAllowed returns the next time after t when actions are permitted.
// If t is not restricted, NextAllowed returns t unchanged.
func (g *Gate) NextAllowed(t time.Time) time.Time {
	if !g.IsRestricted(t) {
		return t
	}
	u := t.UTC()
	// Advance to Sunday 01:00:00 UTC of the same day.
	next := time.Date(u.Year(), u.Month(), u.Day(), 1, 0, 0, 0, time.UTC)
	return next
}

// Check returns ErrRhythmConstraint (wrapped) when t is restricted,
// annotated with the next-allowed time in RFC3339 format.
// Returns nil when the action may proceed.
func (g *Gate) Check(t time.Time) error {
	if !g.IsRestricted(t) {
		return nil
	}
	next := g.NextAllowed(t)
	return fmt.Errorf("%w: retry after %s", ErrRhythmConstraint, next.Format(time.RFC3339))
}
