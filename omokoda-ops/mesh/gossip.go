package mesh

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"
)

// GossipMsg is the payload broadcast during peer announcement.
type GossipMsg struct {
	AgentID    string    `json:"agent_id"`
	BlockID    string    `json:"block_id"`
	Addr       string    `json:"addr"`
	TrustScore float64   `json:"trust_score"`
	Timestamp  time.Time `json:"timestamp"`
}

// Gossiper periodically announces this node's presence to known peers
// and receives announcements from them.
type Gossiper struct {
	self     GossipMsg
	store    *PeerStore
	interval time.Duration
	ttl      time.Duration
	client   *http.Client
}

func NewGossiper(agentID, blockID, selfAddr string, store *PeerStore) *Gossiper {
	return &Gossiper{
		self:     GossipMsg{AgentID: agentID, BlockID: blockID, Addr: selfAddr},
		store:    store,
		interval: 30 * time.Second,
		ttl:      90 * time.Second,
		client:   &http.Client{Timeout: 5 * time.Second},
	}
}

// Run starts the gossip loop until ctx is cancelled.
func (g *Gossiper) Run(ctx context.Context) {
	ticker := time.NewTicker(g.interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			g.self.Timestamp = time.Now()
			evicted := g.store.Evict(g.ttl)
			if evicted > 0 {
				log.Printf("[mesh/gossip] evicted %d stale peers", evicted)
			}
			g.announceToAll()
		}
	}
}

func (g *Gossiper) announceToAll() {
	peers := g.store.All()
	if len(peers) == 0 {
		return
	}
	body, err := json.Marshal(g.self)
	if err != nil {
		return
	}
	for _, p := range peers {
		if p.AgentID == g.self.AgentID {
			continue
		}
		url := fmt.Sprintf("%s/v1/mesh/gossip", p.Addr)
		req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
		if err != nil {
			continue
		}
		req.Header.Set("Content-Type", "application/json")
		resp, err := g.client.Do(req)
		if err != nil {
			log.Printf("[mesh/gossip] announce to %s: %v", p.AgentID, err)
			continue
		}
		resp.Body.Close()
	}
}

// ReceiveAnnouncement processes an incoming gossip message from another peer.
func (g *Gossiper) ReceiveAnnouncement(msg GossipMsg) {
	g.store.Upsert(Peer{
		AgentID:    msg.AgentID,
		BlockID:    msg.BlockID,
		Addr:       msg.Addr,
		TrustScore: msg.TrustScore,
	})
}
