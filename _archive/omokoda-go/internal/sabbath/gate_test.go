package sabbath_test

import (
	"errors"
	"testing"
	"time"

	"github.com/omo-koda/omokoda-go/internal/sabbath"
)

func sunday(hour, min int) time.Time {
	// 2024-01-07 is a known Sunday in UTC.
	return time.Date(2024, 1, 7, hour, min, 0, 0, time.UTC)
}

func monday(hour, min int) time.Time {
	// 2024-01-08 is the Monday following that Sunday.
	return time.Date(2024, 1, 8, hour, min, 0, 0, time.UTC)
}

func TestIsRestricted_Sunday_00_30_restricted(t *testing.T) {
	g := sabbath.New()
	ts := sunday(0, 30)
	if !g.IsRestricted(ts) {
		t.Errorf("expected Sunday 00:30 UTC to be restricted, got unrestricted")
	}
}

func TestIsRestricted_Sunday_01_01_not_restricted(t *testing.T) {
	g := sabbath.New()
	ts := sunday(1, 1)
	if g.IsRestricted(ts) {
		t.Errorf("expected Sunday 01:01 UTC to be unrestricted, got restricted")
	}
}

func TestIsRestricted_Monday_00_30_not_restricted(t *testing.T) {
	g := sabbath.New()
	ts := monday(0, 30)
	if g.IsRestricted(ts) {
		t.Errorf("expected Monday 00:30 UTC to be unrestricted, got restricted")
	}
}

func TestNextAllowed_during_restriction(t *testing.T) {
	g := sabbath.New()
	ts := sunday(0, 30)
	next := g.NextAllowed(ts)
	want := time.Date(2024, 1, 7, 1, 0, 0, 0, time.UTC)
	if !next.Equal(want) {
		t.Errorf("NextAllowed(%v) = %v, want %v", ts, next, want)
	}
}

func TestNextAllowed_outside_restriction(t *testing.T) {
	g := sabbath.New()
	ts := sunday(1, 1)
	next := g.NextAllowed(ts)
	if !next.Equal(ts) {
		t.Errorf("NextAllowed outside restriction should return t unchanged, got %v", next)
	}
}

func TestCheck_returns_error_during_restriction(t *testing.T) {
	g := sabbath.New()
	ts := sunday(0, 45)
	err := g.Check(ts)
	if err == nil {
		t.Fatal("expected error during Sabbath window, got nil")
	}
	if !errors.Is(err, sabbath.ErrRhythmConstraint) {
		t.Errorf("expected ErrRhythmConstraint, got %v", err)
	}
}

func TestCheck_returns_nil_outside_restriction(t *testing.T) {
	g := sabbath.New()
	ts := monday(12, 0)
	if err := g.Check(ts); err != nil {
		t.Errorf("expected nil outside restriction, got %v", err)
	}
}
