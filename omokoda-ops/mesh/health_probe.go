package mesh

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"
)

// ProbeResult records the outcome of a single health probe.
type ProbeResult struct {
	AgentID   string
	Reachable bool
	Latency   time.Duration
	CheckedAt time.Time
}

// HealthProber checks the liveness of known mesh peers.
type HealthProber struct {
	store    *PeerStore
	interval time.Duration
	client   *http.Client
	results  map[string]ProbeResult
}

func NewHealthProber(store *PeerStore) *HealthProber {
	return &HealthProber{
		store:    store,
		interval: 60 * time.Second,
		client:   &http.Client{Timeout: 3 * time.Second},
		results:  make(map[string]ProbeResult),
	}
}

// Run starts periodic health probes until ctx is cancelled.
func (h *HealthProber) Run(ctx context.Context) {
	ticker := time.NewTicker(h.interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			h.probeAll()
		}
	}
}

func (h *HealthProber) probeAll() {
	for _, peer := range h.store.All() {
		result := h.probe(peer)
		h.results[peer.AgentID] = result
		if !result.Reachable {
			log.Printf("[mesh/health] peer %s unreachable (latency: %v)", peer.AgentID, result.Latency)
		}
	}
}

func (h *HealthProber) probe(peer Peer) ProbeResult {
	start := time.Now()
	url := fmt.Sprintf("%s/v1/health", peer.Addr)
	resp, err := h.client.Get(url)
	latency := time.Since(start)
	if err != nil || resp.StatusCode >= 400 {
		if resp != nil {
			resp.Body.Close()
		}
		return ProbeResult{AgentID: peer.AgentID, Reachable: false, Latency: latency, CheckedAt: time.Now()}
	}
	resp.Body.Close()
	return ProbeResult{AgentID: peer.AgentID, Reachable: true, Latency: latency, CheckedAt: time.Now()}
}

// Results returns the latest probe results for all peers.
func (h *HealthProber) Results() map[string]ProbeResult {
	out := make(map[string]ProbeResult, len(h.results))
	for k, v := range h.results {
		out[k] = v
	}
	return out
}
