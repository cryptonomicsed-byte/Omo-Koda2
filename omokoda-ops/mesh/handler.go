package mesh

import (
	"encoding/json"
	"net/http"
)

// Handler wires the mesh PeerStore and Gossiper into HTTP routes.
type Handler struct {
	store    *PeerStore
	gossiper *Gossiper
}

// NewHandler creates a Handler backed by the given store and gossiper.
func NewHandler(store *PeerStore, gossiper *Gossiper) *Handler {
	return &Handler{store: store, gossiper: gossiper}
}

// RegisterRoutes mounts the mesh API under the given ServeMux.
func (h *Handler) RegisterRoutes(mux *http.ServeMux) {
	mux.HandleFunc("/v1/mesh/gossip", h.handleGossip)
	mux.HandleFunc("/mesh/peers", h.handlePeers)
	mux.HandleFunc("/mesh/resource", h.handleResource)
	mux.HandleFunc("/mesh/health", h.handleHealth)
}

<<<<<<< HEAD
=======
// handleGossip accepts incoming gossip announcements from peer agents.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
func (h *Handler) handleGossip(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var msg GossipMsg
	if err := json.NewDecoder(r.Body).Decode(&msg); err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}
	h.gossiper.ReceiveAnnouncement(msg)
	w.WriteHeader(http.StatusNoContent)
}

<<<<<<< HEAD
=======
// handlePeers returns the list of known mesh peers.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
func (h *Handler) handlePeers(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	peers := h.store.All()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{"peers": peers}) //nolint:errcheck
}

<<<<<<< HEAD
=======
// handleResource accepts a resource offer advertisement and registers the peer.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
func (h *Handler) handleResource(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var offer struct {
		AgentID    string  `json:"agent_id"`
		ResourceID string  `json:"resource_id"`
		Kind       string  `json:"kind"`
		Capacity   float64 `json:"capacity"`
	}
	if err := json.NewDecoder(r.Body).Decode(&offer); err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}
	h.store.Upsert(Peer{
		AgentID: offer.AgentID,
		BlockID: offer.Kind,
		Addr:    "",
	})
	w.WriteHeader(http.StatusNoContent)
}

<<<<<<< HEAD
=======
// handleHealth returns basic mesh health metrics.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
func (h *Handler) handleHealth(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	count := h.store.Count()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{ //nolint:errcheck
		"healthy":    true,
		"peer_count": count,
	})
}
