package flow

import (
	"encoding/json"
	"net/http"
	"strings"
	"sync"
	"time"
)

// PrimitiveStore tracks per-agent cooldowns for distributed rhythm enforcement.
// After a primitive is recorded, a brief cooldown is set to prevent
// same-agent bursts across distributed Rust instances.
type PrimitiveStore struct {
	mu        sync.Mutex
	cooldowns map[string]time.Time // agent_id -> cooldown expiry
}

func NewPrimitiveStore() *PrimitiveStore {
	return &PrimitiveStore{cooldowns: make(map[string]time.Time)}
}

// SetCooldown sets a cooldown for agentID lasting d from now.
func (ps *PrimitiveStore) SetCooldown(agentID string, d time.Duration) {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	ps.cooldowns[agentID] = time.Now().Add(d)
}

// IsInCooldown returns true if agentID has an active cooldown.
func (ps *PrimitiveStore) IsInCooldown(agentID string) bool {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	exp, ok := ps.cooldowns[agentID]
	return ok && time.Now().Before(exp)
}

// cooldownDuration returns the rhythm cooldown for a primitive name.
// Irreversible primitives (act) get a longer cooldown than think.
func cooldownDuration(primitive string) time.Duration {
	switch strings.ToLower(primitive) {
	case "act":
		return 2 * time.Second
	case "think":
		return 1 * time.Second
	default:
		return 1 * time.Second
	}
}

// NewHTTPHandler returns an http.Handler for the ỌYA REST API:
//
//	GET  /cooldown/{agent_id}  → {"in_cooldown": bool}
//	POST /record               ← {"agent_id": "…", "primitive": "…"}
func NewHTTPHandler(store *PrimitiveStore) http.Handler {
	mux := http.NewServeMux()

	// GET /cooldown/{agent_id}
	mux.HandleFunc("/cooldown/", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		agentID := strings.TrimPrefix(r.URL.Path, "/cooldown/")
		if agentID == "" {
			http.Error(w, "agent_id required", http.StatusBadRequest)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]bool{"in_cooldown": store.IsInCooldown(agentID)})
	})

	// POST /record
	mux.HandleFunc("/record", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		var req struct {
			AgentID   string `json:"agent_id"`
			Primitive string `json:"primitive"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		if req.AgentID == "" {
			http.Error(w, "agent_id required", http.StatusBadRequest)
			return
		}
		store.SetCooldown(req.AgentID, cooldownDuration(req.Primitive))
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]bool{"recorded": true})
	})

	// GET /health
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]bool{"ok": true})
	})

	return mux
}
