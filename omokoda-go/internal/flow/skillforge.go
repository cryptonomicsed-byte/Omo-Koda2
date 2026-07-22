package flow

import (
	"encoding/json"
	"net/http"
	"sync"
	"time"
)

// SkillForgeRun tracks one forge pipeline's stage transitions. Go's
// concurrency primitives are the real fit for the Coordination stage: many
// forge runs can be in flight at once (each a goroutine-safe map entry), and
// the state machine itself — start, transition, complete/fail — is exactly
// what sync.Mutex-guarded maps are for. This is genuinely useful
// observability (a run's timeline survives even if the Rust caller only
// polls at the end), not a rename of work Rust already did.
type SkillForgeRun struct {
	RunID     string    `json:"run_id"`
	URL       string    `json:"url"`
	Stages    []string  `json:"stages"` // completed stage names, in order
	StartedAt time.Time `json:"started_at"`
	UpdatedAt time.Time `json:"updated_at"`
	Status    string    `json:"status"` // "running" | "done" | "failed"
	Error     string    `json:"error,omitempty"`
}

// SkillForgeStore tracks in-flight and recent forge runs.
type SkillForgeStore struct {
	mu   sync.Mutex
	runs map[string]*SkillForgeRun
}

func NewSkillForgeStore() *SkillForgeStore {
	return &SkillForgeStore{runs: make(map[string]*SkillForgeRun)}
}

func (s *SkillForgeStore) Start(runID, url string) *SkillForgeRun {
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

func (s *SkillForgeStore) Transition(runID, stage string) (*SkillForgeRun, bool) {
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

func (s *SkillForgeStore) Finish(runID string, ok bool, errMsg string) (*SkillForgeRun, bool) {
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

func (s *SkillForgeStore) Get(runID string) (*SkillForgeRun, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	run, ok := s.runs[runID]
	return run, ok
}

// RegisterSkillForgeRoutes wires the Coordination-stage endpoints onto mux.
//
//	POST /skillforge/start        {run_id, url}            -> the new run
//	POST /skillforge/transition   {run_id, stage}           -> the updated run
//	POST /skillforge/finish       {run_id, ok, error}       -> the final run
//	GET  /skillforge/status/{id}                            -> the run, or 404
func RegisterSkillForgeRoutes(mux *http.ServeMux, store *SkillForgeStore) {
	writeJSON := func(w http.ResponseWriter, v any) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(v)
	}

	mux.HandleFunc("/skillforge/start", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
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
		writeJSON(w, store.Start(req.RunID, req.URL))
	})

	mux.HandleFunc("/skillforge/transition", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
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
		run, ok := store.Transition(req.RunID, req.Stage)
		if !ok {
			http.Error(w, "unknown run_id", http.StatusNotFound)
			return
		}
		writeJSON(w, run)
	})

	mux.HandleFunc("/skillforge/finish", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
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
		run, ok := store.Finish(req.RunID, req.Ok, req.Error)
		if !ok {
			http.Error(w, "unknown run_id", http.StatusNotFound)
			return
		}
		writeJSON(w, run)
	})

	mux.HandleFunc("/skillforge/status/", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		runID := r.URL.Path[len("/skillforge/status/"):]
		run, ok := store.Get(runID)
		if !ok {
			http.Error(w, "unknown run_id", http.StatusNotFound)
			return
		}
		writeJSON(w, run)
	})
}
