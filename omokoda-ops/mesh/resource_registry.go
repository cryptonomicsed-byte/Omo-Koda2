package mesh

import (
	"sync"
	"time"
)

// ResourceOffer is a resource this agent makes available to the mesh.
type ResourceOffer struct {
	ResourceID string
	Kind       string // "compute", "storage", "bandwidth", "data"
	Capacity   float64
	ExpiresAt  time.Time
	OwnerID    string
}

// ResourceRegistry is a thread-safe in-memory board of resource offers.
type ResourceRegistry struct {
	mu      sync.RWMutex
	offers  map[string]*ResourceOffer
}

func NewResourceRegistry() *ResourceRegistry {
	return &ResourceRegistry{offers: make(map[string]*ResourceOffer)}
}

// Register adds or replaces a resource offer.
func (r *ResourceRegistry) Register(offer ResourceOffer) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.offers[offer.ResourceID] = &offer
}

// Available returns all non-expired offers, optionally filtered by kind.
func (r *ResourceRegistry) Available(kind string) []ResourceOffer {
	r.mu.RLock()
	defer r.mu.RUnlock()
	now := time.Now()
	var out []ResourceOffer
	for _, o := range r.offers {
		if o.ExpiresAt.Before(now) {
			continue
		}
		if kind != "" && o.Kind != kind {
			continue
		}
		out = append(out, *o)
	}
	return out
}

// Remove deletes a resource offer.
func (r *ResourceRegistry) Remove(resourceID string) {
	r.mu.Lock()
	defer r.mu.Unlock()
	delete(r.offers, resourceID)
}

// Expire purges stale offers.
func (r *ResourceRegistry) Expire() int {
	r.mu.Lock()
	defer r.mu.Unlock()
	now := time.Now()
	removed := 0
	for id, o := range r.offers {
		if o.ExpiresAt.Before(now) {
			delete(r.offers, id)
			removed++
		}
	}
	return removed
}
