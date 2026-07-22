package flow

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestSkillForgeStore_LifecycleIsTracked(t *testing.T) {
	store := NewSkillForgeStore()
	run := store.Start("run-1", "https://github.com/a/b")
	if run.Status != "running" || len(run.Stages) != 0 {
		t.Fatalf("unexpected initial run: %+v", run)
	}

	if _, ok := store.Transition("run-1", "analysis"); !ok {
		t.Fatal("transition on known run should succeed")
	}
	run, _ = store.Transition("run-1", "creation")
	if len(run.Stages) != 2 || run.Stages[0] != "analysis" || run.Stages[1] != "creation" {
		t.Fatalf("stages not recorded in order: %+v", run.Stages)
	}

	if _, ok := store.Transition("unknown-run", "x"); ok {
		t.Fatal("transition on unknown run should fail")
	}

	run, ok := store.Finish("run-1", true, "")
	if !ok || run.Status != "done" {
		t.Fatalf("expected done status, got: %+v", run)
	}
}

func TestSkillForgeStore_FailurePathRecordsError(t *testing.T) {
	store := NewSkillForgeStore()
	store.Start("run-2", "https://github.com/a/b")
	run, ok := store.Finish("run-2", false, "audit gate failed")
	if !ok || run.Status != "failed" || run.Error != "audit gate failed" {
		t.Fatalf("expected failed status with error, got: %+v", run)
	}
}

func TestSkillForgeRoutes_FullFlowOverHTTP(t *testing.T) {
	handler := NewHTTPHandler(NewPrimitiveStore())

	start := httptest.NewRequest(http.MethodPost, "/skillforge/start",
		bytes.NewBufferString(`{"run_id":"r1","url":"https://github.com/a/b"}`))
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, start)
	if rec.Code != http.StatusOK {
		t.Fatalf("start: expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	trans := httptest.NewRequest(http.MethodPost, "/skillforge/transition",
		bytes.NewBufferString(`{"run_id":"r1","stage":"analysis"}`))
	rec = httptest.NewRecorder()
	handler.ServeHTTP(rec, trans)
	if rec.Code != http.StatusOK {
		t.Fatalf("transition: expected 200, got %d", rec.Code)
	}

	status := httptest.NewRequest(http.MethodGet, "/skillforge/status/r1", nil)
	rec = httptest.NewRecorder()
	handler.ServeHTTP(rec, status)
	if rec.Code != http.StatusOK {
		t.Fatalf("status: expected 200, got %d", rec.Code)
	}
	var run SkillForgeRun
	if err := json.Unmarshal(rec.Body.Bytes(), &run); err != nil {
		t.Fatalf("bad status body: %v", err)
	}
	if len(run.Stages) != 1 || run.Stages[0] != "analysis" {
		t.Fatalf("expected 1 stage recorded, got: %+v", run.Stages)
	}

	missing := httptest.NewRequest(http.MethodGet, "/skillforge/status/nope", nil)
	rec = httptest.NewRecorder()
	handler.ServeHTTP(rec, missing)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for unknown run, got %d", rec.Code)
	}
}
