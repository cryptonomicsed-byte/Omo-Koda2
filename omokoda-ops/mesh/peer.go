// Package mesh implements the Block Mesh network layer for Ọmọ Kọ́dà agents.
// It handles peer discovery, gossip, resource registration, and health probing.
package mesh

import (
	"sync"
	"time"
)

// Peer represents a known mesh participant.
type Peer struct {
	AgentID    string
	BlockID    string
<<<<<<< HEAD
	Addr       string
=======
	Addr       string // base URL of the peer's Ọmọ Kọ́dà steward
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
	LastSeen   time.Time
	TrustScore float64
}

// PeerStore is a thread-safe store of known peers.
type PeerStore struct {
	mu    sync.RWMutex
<<<<<<< HEAD
	peers map[string]*Peer
=======
	peers map[string]*Peer // keyed by AgentID
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
}

func NewPeerStore() *PeerStore {
	return &PeerStore{peers: make(map[string]*Peer)}
}

func (s *PeerStore) Upsert(p Peer) {
	s.mu.Lock()
	defer s.mu.Unlock()
	p.LastSeen = time.Now()
	s.peers[p.AgentID] = &p
}

func (s *PeerStore) Get(agentID string) (*Peer, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	p, ok := s.peers[agentID]
	return p, ok
}

func (s *PeerStore) All() []Peer {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make([]Peer, 0, len(s.peers))
	for _, p := range s.peers {
		out = append(out, *p)
	}
	return out
}

// Evict removes peers not seen within ttl.
func (s *PeerStore) Evict(ttl time.Duration) int {
	s.mu.Lock()
	defer s.mu.Unlock()
	cutoff := time.Now().Add(-ttl)
	evicted := 0
	for id, p := range s.peers {
		if p.LastSeen.Before(cutoff) {
			delete(s.peers, id)
			evicted++
		}
	}
	return evicted
}

func (s *PeerStore) Count() int {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return len(s.peers)
}
