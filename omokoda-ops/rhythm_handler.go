package main

import (
	"encoding/json"
	"net/http"
	"time"
)

type rhythmCooldownRequest struct {
	AgentID   string `json:"agent_id"`
	Primitive string `json:"primitive"`
}

type rhythmCooldownResponse struct {
	InCooldown bool `json:"in_cooldown"`
}

type rhythmRecordRequest struct {
	AgentID   string `json:"agent_id"`
	Primitive string `json:"primitive"`
}

<<<<<<< HEAD
=======
// cooldownCheck reports whether any tracked primitive is currently rate-limited
// for the given agent. Read-only: does not update the tracker.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
func cooldownCheck(agentID string) bool {
	tracker.mu.Lock()
	defer tracker.mu.Unlock()
	for tool, cd := range toolCooldowns {
		key := agentID + ":" + tool
		if last, seen := tracker.lastUsed[key]; seen && time.Since(last) < cd {
			_ = tool
			return true
		}
	}
	return false
}

<<<<<<< HEAD
=======
// recordPrimitive stamps the current time for agent+primitive in the tracker.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
func recordPrimitive(agentID, primitive string) {
	if _, ok := toolCooldowns[primitive]; !ok {
		return
	}
	key := agentID + ":" + primitive
	tracker.mu.Lock()
	tracker.lastUsed[key] = time.Now()
	tracker.mu.Unlock()
}

func rhythmCooldownHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var req rhythmCooldownRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"bad request"}`, http.StatusBadRequest)
		return
	}
	resp := rhythmCooldownResponse{InCooldown: cooldownCheck(req.AgentID)}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp) //nolint:errcheck
}

func rhythmRecordHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var req rhythmRecordRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, `{"error":"bad request"}`, http.StatusBadRequest)
		return
	}
	recordPrimitive(req.AgentID, req.Primitive)
	w.WriteHeader(http.StatusNoContent)
}
