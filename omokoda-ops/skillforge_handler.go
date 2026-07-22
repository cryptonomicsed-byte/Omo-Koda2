package main

import (
	"encoding/json"
	"net/http"
	"sync"
	"time"
)

// SkillForgeRun tracks one forge pipeline's stage transitions. Consolidated
// here from the smaller, now-retired omokoda-go/oya service -- omokoda-ops
// is the more complete Ọya implementation (metrics, SSE, device management,
// a full /v1/* proxy, and richer per-tool cooldown tracking than oya ever
// had), so new Ọya capability lands here going forward.
type SkillForgeRun struct {
	RunID     string    `json:"run_id"`
	URL       string    `json:"url"`
	Stages    []string  `json:"stages"`
	StartedAt time.Time `json:"started_at"`
	UpdatedAt time.Time `json:"updated_at"`
	Status    string    `json:"status"`
	Error     string    `json:"error,omitempty"`
}

type skillForgeStore struct {
	mu   sync.Mutex
	runs map[string]*SkillForgeRun
}

var sfStore = &skillForgeStore{runs: make(map[string]*SkillForgeRun)}

func (s *skillForgeStore) start(runID, url string) *SkillForgeRun {
	s.mu.Lock()
	defer s.mu.Unlock()
	now := time.Now()
	run := &SkillForgeRun{
		RunID: runID, URL: url, Stages: []string{},
		StartedAt: now, UpdatedAt: now, Status: "running",
	}
	s.runs[runID] = run
	return run
}

func (s *skillForgeStore) transition(runID, stage string) (*SkillForgeRun, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	run, ok := s.runs[runID]
	if !ok {
		return nil, false
	}
	run.Stages = append(run.Stages, stage)
	run.UpdatedAt = time.Now()
	return run, true
}

func (s *skillForgeStore) finish(runID string, ok bool, errMsg string) (*SkillForgeRun, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	run, found := s.runs[runID]
	if !found {
		return nil, false
	}
	if ok {
		run.Status = "done"
	} else {
		run.Status = "failed"
		run.Error = errMsg
	}
	run.UpdatedAt = time.Now()
	return run, true
}

func (s *skillForgeStore) get(runID string) (*SkillForgeRun, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	run, ok := s.runs[runID]
	return run, ok
}

func skillforgeStartHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var req struct {
		RunID string `json:"run_id"`
		URL   string `json:"url"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.RunID == "" {
		http.Error(w, "run_id required", http.StatusBadRequest)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(sfStore.start(req.RunID, req.URL)) //nolint:errcheck
}

func skillforgeTransitionHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var req struct {
		RunID string `json:"run_id"`
		Stage string `json:"stage"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.RunID == "" {
		http.Error(w, "run_id required", http.StatusBadRequest)
		return
	}
	run, ok := sfStore.transition(req.RunID, req.Stage)
	if !ok {
		http.Error(w, "unknown run_id", http.StatusNotFound)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(run) //nolint:errcheck
}

func skillforgeFinishHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var req struct {
		RunID string `json:"run_id"`
		Ok    bool   `json:"ok"`
		Error string `json:"error"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.RunID == "" {
		http.Error(w, "run_id required", http.StatusBadRequest)
		return
	}
	run, ok := sfStore.finish(req.RunID, req.Ok, req.Error)
	if !ok {
		http.Error(w, "unknown run_id", http.StatusNotFound)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(run) //nolint:errcheck
}

func skillforgeStatusHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	runID := r.URL.Path[len("/skillforge/status/"):]
	run, ok := sfStore.get(runID)
	if !ok {
		http.Error(w, "unknown run_id", http.StatusNotFound)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(run) //nolint:errcheck
}
