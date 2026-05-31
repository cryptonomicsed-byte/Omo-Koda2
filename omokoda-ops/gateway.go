package main

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
)

const defaultStewardURL = "http://localhost:7777"

func getStewardURL() string {
	if url := os.Getenv("STEWARD_URL"); url != "" {
		return url
	}
	return defaultStewardURL
}

// proxyToSteward transparently forwards a request to the Rust Steward and
// streams the response back to the caller.
func proxyToSteward(w http.ResponseWriter, r *http.Request) {
	target := getStewardURL() + r.URL.Path

	req, err := http.NewRequestWithContext(r.Context(), r.Method, target, r.Body)
	if err != nil {
		jsonError(w, "failed to create upstream request", http.StatusBadGateway)
		return
	}
	if ct := r.Header.Get("Content-Type"); ct != "" {
		req.Header.Set("Content-Type", ct)
	}
	req.Header.Set("X-Forwarded-For", r.RemoteAddr)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[gateway] upstream error: %v", err)
		jsonError(w, "steward unreachable", http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	for k, vals := range resp.Header {
		for _, v := range vals {
			w.Header().Add(k, v)
		}
	}
	w.WriteHeader(resp.StatusCode)
	io.Copy(w, resp.Body) //nolint:errcheck
}

// actHandler reads the request body to run the rhythm gate, then proxies to
// the Steward. The body is restored after peeking so the Steward still
// receives it intact.
func actHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		proxyToSteward(w, r)
		return
	}

	body, err := io.ReadAll(r.Body)
	if err != nil {
		jsonError(w, "failed to read body", http.StatusBadRequest)
		return
	}
	// Restore the body for forwarding.
	r.Body = io.NopCloser(bytes.NewReader(body))

	var req struct {
		Tool string `json:"tool"`
	}
	json.Unmarshal(body, &req) //nolint:errcheck

	agentID := r.Header.Get("X-Agent-Id")
	switch CheckRhythm(req.Tool, agentID) {
	case RhythmSabbath:
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusTooManyRequests)
		json.NewEncoder(w).Encode(map[string]string{ //nolint:errcheck
			"error":   "sabbath",
			"message": "Irreversible actions are paused during Sabbath (UTC Saturday)",
		})
		return
	case RhythmCooldown:
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusTooManyRequests)
		json.NewEncoder(w).Encode(map[string]string{ //nolint:errcheck
			"error":   "cooldown",
			"message": "Tool is in cooldown. Try again shortly.",
		})
		return
	}

	proxyToSteward(w, r)
}

func jsonError(w http.ResponseWriter, msg string, code int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	json.NewEncoder(w).Encode(map[string]string{"error": msg}) //nolint:errcheck
}
