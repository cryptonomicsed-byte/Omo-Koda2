package flow

import (
	"encoding/json"
	"net/http"
	"strings"
	"sync"
	"time"
)

// PrimitiveStore tracks per-agent cooldowns for distributed rhythm enforcement.
type PrimitiveStore struct {
	mu        sync.Mutex
	cooldowns map[string]time.Time
}

func NewPrimitiveStore() *PrimitiveStore {
	return &PrimitiveStore{cooldowns: make(map[string]time.Time)}
}

func (ps *PrimitiveStore) SetCooldown(agentID string, d time.Duration) {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	ps.cooldowns[agentID] = time.Now().Add(d)
}

func (ps *PrimitiveStore) IsInCooldown(agentID string) bool {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	exp, ok := ps.cooldowns[agentID]
	return ok && time.Now().Before(exp)
}

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

// corsMiddleware allows the Axiom browser dashboard (a different origin/port)
// to call this service directly, matching the permissive CORS the Rust
// kernel (:7777), LOOM (:8889), the Julia memory service (:7778), and the
// Elixir swarm (:4000) already use.
func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// NewHTTPHandler returns an http.Handler for the ỌYA REST API.
func NewHTTPHandler(store *PrimitiveStore) http.Handler {
	return NewHTTPHandlerWithSkillForge(store, NewSkillForgeStore())
}

// NewHTTPHandlerWithSkillForge is NewHTTPHandler plus the SkillForge
// Coordination-stage routes, for callers that want to share/inspect the
// SkillForgeStore (e.g. tests).
func NewHTTPHandlerWithSkillForge(store *PrimitiveStore, sfStore *SkillForgeStore) http.Handler {
	mux := http.NewServeMux()

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

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]bool{"ok": true})
	})

	RegisterSkillForgeRoutes(mux, sfStore)

	return corsMiddleware(mux)
}
