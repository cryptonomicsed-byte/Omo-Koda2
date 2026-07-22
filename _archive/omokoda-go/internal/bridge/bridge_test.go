package bridge

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// newTestBridge returns an OrishaBridge whose all service URLs point to the
// provided httptest.Server. Individual tests override specific URLs as needed.
func newTestBridge(srv *httptest.Server) *OrishaBridge {
	cfg := ServiceConfig{
		StewardURL: srv.URL,
		OsunURL:    srv.URL,
		OgunURL:    srv.URL,
		ObatalaURL: srv.URL,
	}
	return NewBridge(cfg)
}

func TestRouteThink_Success(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/think" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"content":"hello","iris_profile":"balanced","blocked":false}`))
	}))
	defer srv.Close()

	b := newTestBridge(srv)
	req := ThinkRequest{
		AgentID: "agent-1",
		Prompt:  "What is the nature of ase?",
		Emotion: EmotionSnapshot{Energy: 0.8, Focus: 0.9},
	}

	resp, err := b.RouteThink(context.Background(), req)
	if err != nil {
		t.Fatalf("RouteThink returned unexpected error: %v", err)
	}
	if resp.Content != "hello" {
		t.Errorf("expected content %q, got %q", "hello", resp.Content)
	}
	if resp.IrisProfile != "balanced" {
		t.Errorf("expected iris_profile %q, got %q", "balanced", resp.IrisProfile)
	}
	if resp.Blocked {
		t.Error("expected blocked=false, got true")
	}
}

func TestRouteHermetic_Blocked(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/hermetic/eval" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"allowed":false,"decision":"Block: vibration","scores":{},"overall":0.3}`))
	}))
	defer srv.Close()

	b := newTestBridge(srv)
	req := HermeticRequest{
		Intent:  "harm",
		Action:  "manipulate",
		AgentID: "agent-2",
		Emotion: EmotionSnapshot{Tension: 0.95},
	}

	resp, err := b.RouteHermetic(context.Background(), req)
	if err != nil {
		t.Fatalf("RouteHermetic returned unexpected error: %v", err)
	}
	if resp.Allowed {
		t.Error("expected allowed=false, got true")
	}
	if resp.Decision != "Block: vibration" {
		t.Errorf("expected decision %q, got %q", "Block: vibration", resp.Decision)
	}
	if resp.Overall != 0.3 {
		t.Errorf("expected overall=0.3, got %v", resp.Overall)
	}
}

func TestRouteMemoryReconstruct_ReturnsContext(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/soma/reconstruct" {
			t.Errorf("unexpected path: %s", r.URL.Path)
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"context":"agent-3 remembered the river","lpm_summary":"river crossing, cycle 42"}`))
	}))
	defer srv.Close()

	b := newTestBridge(srv)
	req := MemoryReconstructRequest{
		AgentID: "agent-3",
		Query:   "river",
		Emotion: EmotionSnapshot{Connection: 0.7},
	}

	resp, err := b.RouteMemoryReconstruct(context.Background(), req)
	if err != nil {
		t.Fatalf("RouteMemoryReconstruct returned unexpected error: %v", err)
	}
	if resp.Context == "" {
		t.Error("expected non-empty context in response")
	}
	if resp.LPMSummary == "" {
		t.Error("expected non-empty lpm_summary in response")
	}
}

func TestPost_NonOkStatus(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer srv.Close()

	b := newTestBridge(srv)
	// Use post directly via RouteThink so we exercise the error path end-to-end.
	req := ThinkRequest{AgentID: "agent-err", Prompt: "oops"}
	_, err := b.RouteThink(context.Background(), req)
	if err == nil {
		t.Fatal("expected error for 500 response, got nil")
	}
	if !strings.Contains(err.Error(), "500") {
		t.Errorf("expected error message to contain %q, got: %v", "500", err)
	}
}
