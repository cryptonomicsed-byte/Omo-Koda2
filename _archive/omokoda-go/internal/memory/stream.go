// Package memory implements the Ọya memory streaming layer.
//
// omo-mem pattern mapping:
//   memory_list   → StreamManager.List()
//   memory_append → StreamManager.Append()
//   memory_read   → MemoryStream.Search() / Entries()
//
// Streams are append-only per-agent daily logs.  Content is not indexed
// externally — Search does a linear scan; semantic recall is delegated to
// the Julia RACK via the Ọ̀ṣun service bridge.
package memory

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"
)

// StreamEntry is one appended memory note.
type StreamEntry struct {
	Timestamp time.Time
	Content   string
	Tags      []string
	Sequence  uint64
}

// MemoryStream is an append-only log for one agent on one calendar day.
type MemoryStream struct {
	AgentID  string
	StreamID string // "agentID:daily:YYYY-MM-DD"
	entries  []StreamEntry
	mu       sync.RWMutex
	seq      uint64
}

// Append adds a note to the stream. Thread-safe.
func (s *MemoryStream) Append(content string, tags []string) StreamEntry {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.seq++
	e := StreamEntry{
		Timestamp: time.Now().UTC(),
		Content:   content,
		Tags:      tags,
		Sequence:  s.seq,
	}
	s.entries = append(s.entries, e)
	return e
}

// Entries returns a snapshot of all entries. Thread-safe.
func (s *MemoryStream) Entries() []StreamEntry {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := make([]StreamEntry, len(s.entries))
	copy(out, s.entries)
	return out
}

// Search returns entries whose content contains query (case-insensitive).
func (s *MemoryStream) Search(query string) []StreamEntry {
	s.mu.RLock()
	defer s.mu.RUnlock()
	q := strings.ToLower(query)
	var results []StreamEntry
	for _, e := range s.entries {
		if strings.Contains(strings.ToLower(e.Content), q) {
			results = append(results, e)
		}
	}
	return results
}

// LastActivity returns the timestamp of the most recent entry, or zero.
func (s *MemoryStream) LastActivity() time.Time {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if len(s.entries) == 0 {
		return time.Time{}
	}
	return s.entries[len(s.entries)-1].Timestamp
}

// StreamManager manages per-agent daily streams.
type StreamManager struct {
	streams map[string]*MemoryStream
	mu      sync.RWMutex
}

// NewStreamManager creates an empty StreamManager.
func NewStreamManager() *StreamManager {
	return &StreamManager{streams: make(map[string]*MemoryStream)}
}

func streamKey(agentID string, date time.Time) string {
	return fmt.Sprintf("%s:daily:%s", agentID, date.Format("2006-01-02"))
}

// EnsureStream returns the stream for agentID on date, creating it if absent.
func (m *StreamManager) EnsureStream(agentID string, date time.Time) *MemoryStream {
	key := streamKey(agentID, date)

	m.mu.RLock()
	if s, ok := m.streams[key]; ok {
		m.mu.RUnlock()
		return s
	}
	m.mu.RUnlock()

	m.mu.Lock()
	defer m.mu.Unlock()
	// Double-check after write lock
	if s, ok := m.streams[key]; ok {
		return s
	}
	s := &MemoryStream{AgentID: agentID, StreamID: key}
	m.streams[key] = s
	return s
}

// TodayStream returns today's stream for agentID.
func (m *StreamManager) TodayStream(agentID string) *MemoryStream {
	return m.EnsureStream(agentID, time.Now())
}

// Append adds a note to today's stream for agentID.
func (m *StreamManager) Append(agentID, content string, tags []string) StreamEntry {
	return m.TodayStream(agentID).Append(content, tags)
}

// List returns date strings (YYYY-MM-DD) for all streams belonging to agentID.
func (m *StreamManager) List(agentID string) []string {
	m.mu.RLock()
	defer m.mu.RUnlock()
	prefix := agentID + ":daily:"
	var dates []string
	for key := range m.streams {
		if strings.HasPrefix(key, prefix) {
			dates = append(dates, strings.TrimPrefix(key, prefix))
		}
	}
	return dates
}

// Prune removes streams whose stream date is older than maxAge.
// The stream date is parsed from the key suffix "agentID:daily:YYYY-MM-DD".
// Returns the count of streams removed.
func (m *StreamManager) Prune(_ context.Context, maxAge time.Duration) int {
	cutoff := time.Now().Add(-maxAge)
	m.mu.Lock()
	defer m.mu.Unlock()
	removed := 0
	for key := range m.streams {
		parts := strings.SplitN(key, ":daily:", 2)
		if len(parts) != 2 {
			continue
		}
		streamDate, err := time.Parse("2006-01-02", parts[1])
		if err != nil {
			continue
		}
		if streamDate.Before(cutoff) {
			delete(m.streams, key)
			removed++
		}
	}
	return removed
}
