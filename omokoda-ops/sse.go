package main

import (
	"bufio"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"
)

// SSEHub connects to the Rust Steward's SSE stream and fans the events out
// to any number of downstream HTTP clients. It reconnects automatically if
// the upstream disappears.
type SSEHub struct {
	mu         sync.RWMutex
	clients    map[chan string]struct{}
	upstreamURL string
}

// NewSSEHub creates a hub that will relay events from upstreamURL.
func NewSSEHub(upstreamURL string) *SSEHub {
	return &SSEHub{
		clients:     make(map[chan string]struct{}),
		upstreamURL: upstreamURL,
	}
}

func (h *SSEHub) subscribe() chan string {
	ch := make(chan string, 64)
	h.mu.Lock()
	h.clients[ch] = struct{}{}
	h.mu.Unlock()
	return ch
}

func (h *SSEHub) unsubscribe(ch chan string) {
	h.mu.Lock()
	delete(h.clients, ch)
	h.mu.Unlock()
	close(ch)
}

func (h *SSEHub) broadcast(line string) {
	h.mu.RLock()
	defer h.mu.RUnlock()
	for ch := range h.clients {
		select {
		case ch <- line:
		default:
			// Slow consumer — drop rather than block the broadcaster.
		}
	}
}

// Run loops forever: connect to the upstream, relay lines, reconnect on failure.
func (h *SSEHub) Run() {
	backoff := 2 * time.Second
	for {
		if err := h.relay(); err != nil {
			log.Printf("[sse-hub] upstream error: %v — retrying in %s", err, backoff)
		}
		time.Sleep(backoff)
		if backoff < 30*time.Second {
			backoff *= 2
		}
	}
}

func (h *SSEHub) relay() error {
	resp, err := http.Get(h.upstreamURL)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	// Reset backoff on successful connection.
	scanner := bufio.NewScanner(resp.Body)
	for scanner.Scan() {
		h.broadcast(scanner.Text())
	}
	return scanner.Err()
}

// ServeHTTP implements http.Handler so the hub can be registered directly as
// a route target. Each incoming client gets its own channel.
func (h *SSEHub) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "streaming not supported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	ch := h.subscribe()
	defer h.unsubscribe(ch)

	// Initial keepalive so the client knows it's connected.
	fmt.Fprintf(w, ": connected\n\n")
	flusher.Flush()

	keepalive := time.NewTicker(15 * time.Second)
	defer keepalive.Stop()

	for {
		select {
		case line, ok := <-ch:
			if !ok {
				return
			}
			fmt.Fprintf(w, "%s\n", line)
			flusher.Flush()
		case <-keepalive.C:
			fmt.Fprintf(w, ": keepalive\n\n")
			flusher.Flush()
		case <-r.Context().Done():
			return
		}
	}
}
