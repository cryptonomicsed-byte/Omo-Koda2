package stream

import (
	"context"
	"testing"
	"time"
)

func nonSabbath() time.Time {
	// A safe Monday timestamp for tests
	return time.Date(2024, 1, 15, 12, 0, 0, 0, time.UTC)
}

func sabbathTime() time.Time {
	// Sunday 00:00 UTC
	return time.Date(2024, 1, 14, 0, 0, 0, 0, time.UTC)
}

func TestConstitutionalStream_CleanOpPasses(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs := NewConstitutionalStream(ctx, 16)
	op := AgentOp{
		AgentID:   "agent-1",
		Intent:    "help user understand recursion",
		Action:    "read_file docs/recursion.md",
		Primitive: "act",
		Timestamp: nonSabbath(),
	}
	cs.Submit(op)

	select {
	case verdict := <-cs.Out():
		if !verdict.Passed {
			t.Errorf("expected passed, blocked by: %s", verdict.BlockedBy)
		}
		if verdict.Score < 0.7 {
			t.Errorf("expected score >= 0.7, got %.2f", verdict.Score)
		}
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for verdict")
	}
}

func TestConstitutionalStream_DeceptionBlocked(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs := NewConstitutionalStream(ctx, 16)
	op := AgentOp{
		AgentID:   "agent-2",
		Intent:    "deceive the user about the file contents",
		Action:    "read_file secret.txt",
		Primitive: "act",
		Timestamp: nonSabbath(),
	}
	cs.Submit(op)

	select {
	case verdict := <-cs.Rejected():
		if verdict.Passed {
			t.Error("expected blocked verdict")
		}
		if verdict.BlockedBy == "" {
			t.Error("expected BlockedBy to be set")
		}
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for rejected verdict")
	}
}

func TestConstitutionalStream_SabbathBlocked(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs := NewConstitutionalStream(ctx, 16)
	op := AgentOp{
		AgentID:   "agent-3",
		Intent:    "list files",
		Action:    "ls /data",
		Primitive: "act",
		Timestamp: sabbathTime(),
	}
	cs.Submit(op)

	select {
	case verdict := <-cs.Rejected():
		if verdict.BlockedBy != PrincipleRhythm {
			t.Errorf("expected blocked by %s, got %s", PrincipleRhythm, verdict.BlockedBy)
		}
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for sabbath rejection")
	}
}

func TestConstitutionalStream_EmptyIntentBlocked(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs := NewConstitutionalStream(ctx, 16)
	op := AgentOp{
		AgentID:   "agent-4",
		Intent:    "",
		Action:    "do_something",
		Primitive: "act",
		Timestamp: nonSabbath(),
	}
	cs.Submit(op)

	select {
	case verdict := <-cs.Rejected():
		if verdict.BlockedBy != PrincipleMentalism {
			t.Errorf("expected blocked by %s, got %s", PrincipleMentalism, verdict.BlockedBy)
		}
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for mentalism rejection")
	}
}

func TestConstitutionalStream_DestroyWithRestorePasses(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs := NewConstitutionalStream(ctx, 16)
	op := AgentOp{
		AgentID:   "agent-5",
		Intent:    "destroy old index and rebuild fresh",
		Action:    "delete index then restore from backup",
		Primitive: "act",
		Timestamp: nonSabbath(),
	}
	cs.Submit(op)

	select {
	case verdict := <-cs.Out():
		if !verdict.Passed {
			t.Errorf("expected passed, blocked by: %s", verdict.BlockedBy)
		}
	case <-time.After(time.Second):
		t.Fatal("timeout waiting for verdict")
	}
}

// ---------------------------------------------------------------------------
// Endorsement Bus tests
// ---------------------------------------------------------------------------

func TestEndorsementBus_AggregateScoreNeutralWhenEmpty(t *testing.T) {
	eb := NewEndorsementBus(16)
	score := eb.AggregateScore("agent-x", PrincipleMentalism)
	if score != 0.5 {
		t.Errorf("expected 0.5, got %.2f", score)
	}
}

func TestEndorsementBus_EndorseAndAggregate(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	eb := NewEndorsementBus(16)
	go eb.Drain(ctx)

	eb.Endorse("agent-a", "agent-b", PrincipleMentalism, 0.90, "clear intent observed")
	eb.Endorse("agent-c", "agent-b", PrincipleMentalism, 0.80, "consistent declarations")

	// Give the drain goroutine time to process
	time.Sleep(20 * time.Millisecond)

	score := eb.AggregateScore("agent-b", PrincipleMentalism)
	if score < 0.84 || score > 0.86 {
		t.Errorf("expected ~0.85, got %.3f", score)
	}
}

func TestEndorsementBus_PeerReviewNoEndorsements(t *testing.T) {
	eb := NewEndorsementBus(16)
	review := eb.PeerReview("agent-x", "agent-y")
	if review == "" {
		t.Error("expected non-empty review")
	}
}

func TestEndorsementBus_RecentEndorsementsLimit(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	eb := NewEndorsementBus(32)
	go eb.Drain(ctx)

	for i := 0; i < 10; i++ {
		eb.Endorse("a", "b", PrincipleCorrespondence, 0.8, "ok")
	}
	time.Sleep(30 * time.Millisecond)

	recent := eb.RecentEndorsements("b", 3)
	if len(recent) != 3 {
		t.Errorf("expected 3 recent endorsements, got %d", len(recent))
	}
}
