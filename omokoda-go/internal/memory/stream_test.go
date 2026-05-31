package memory_test

import (
	"context"
	"testing"
	"time"

	"github.com/omo-koda/oya/internal/memory"
)

func TestAppendAndSearch(t *testing.T) {
	mgr := memory.NewStreamManager()
	mgr.Append("agent-1", "constitutional alignment confirmed", nil)
	mgr.Append("agent-1", "hermetic vibration principle applied", nil)

	stream := mgr.TodayStream("agent-1")
	results := stream.Search("hermetic")
	if len(results) != 1 {
		t.Fatalf("expected 1 result, got %d", len(results))
	}
	if results[0].Content != "hermetic vibration principle applied" {
		t.Errorf("unexpected content: %s", results[0].Content)
	}
}

func TestList(t *testing.T) {
	mgr := memory.NewStreamManager()
	mgr.Append("agent-2", "note one", []string{"identity"})
	mgr.Append("agent-2", "note two", nil)

	dates := mgr.List("agent-2")
	if len(dates) != 1 {
		t.Fatalf("expected 1 date, got %d", len(dates))
	}
	today := time.Now().UTC().Format("2006-01-02")
	if dates[0] != today {
		t.Errorf("expected %s, got %s", today, dates[0])
	}
}

func TestEntriesSnapshot(t *testing.T) {
	mgr := memory.NewStreamManager()
	mgr.Append("agent-3", "first note", nil)
	mgr.Append("agent-3", "second note", nil)

	entries := mgr.TodayStream("agent-3").Entries()
	if len(entries) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(entries))
	}
	if entries[0].Sequence != 1 || entries[1].Sequence != 2 {
		t.Errorf("unexpected sequences: %d, %d", entries[0].Sequence, entries[1].Sequence)
	}
}

func TestPrune(t *testing.T) {
	mgr := memory.NewStreamManager()
	// Create a stream for yesterday
	yesterday := time.Now().Add(-25 * time.Hour)
	stream := mgr.EnsureStream("agent-4", yesterday)
	stream.Append("old note", nil)

	// Prune streams older than 1 day
	removed := mgr.Prune(context.Background(), 24*time.Hour)
	if removed != 1 {
		t.Errorf("expected 1 removed, got %d", removed)
	}
}

func TestListIsolatesAgents(t *testing.T) {
	mgr := memory.NewStreamManager()
	mgr.Append("agent-a", "agent a note", nil)
	mgr.Append("agent-b", "agent b note", nil)

	datesA := mgr.List("agent-a")
	datesB := mgr.List("agent-b")

	if len(datesA) != 1 || len(datesB) != 1 {
		t.Errorf("expected 1 date each, got %d and %d", len(datesA), len(datesB))
	}
}
