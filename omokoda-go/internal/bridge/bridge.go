package bridge

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// ServiceConfig holds base URLs for each Orisha service.
type ServiceConfig struct {
	StewardURL string // Rust — omokoda-core HTTP API
	OsunURL    string // Julia — SOMA memory service
	OgunURL    string // Python — LLM + tool execution
	ObatalaURL string // Lisp — hermetic ethics
}

// DefaultConfig returns service URLs suitable for local development.
func DefaultConfig() ServiceConfig {
	return ServiceConfig{
		StewardURL: "http://localhost:8080",
		OsunURL:    "http://localhost:8081",
		OgunURL:    "http://localhost:8082",
		ObatalaURL: "http://localhost:8083",
	}
}

// OrishaBridge routes requests between all language services.
type OrishaBridge struct {
	config ServiceConfig
	client *http.Client
}

// NewBridge constructs an OrishaBridge with a 15-second HTTP client timeout.
func NewBridge(config ServiceConfig) *OrishaBridge {
	return &OrishaBridge{
		config: config,
		client: &http.Client{Timeout: 15 * time.Second},
	}
}

// EmotionSnapshot carries the agent's current emotional state dimensions.
type EmotionSnapshot struct {
	Energy     float32 `json:"energy"`
	Tension    float32 `json:"tension"`
	Connection float32 `json:"connection"`
	Focus      float32 `json:"focus"`
}

// ThinkRequest is sent to the steward (Rust) to route a think primitive.
type ThinkRequest struct {
	AgentID string          `json:"agent_id"`
	Prompt  string          `json:"prompt"`
	Private bool            `json:"private"`
	Emotion EmotionSnapshot `json:"emotion"`
}

// ThinkResponse is the result returned from the steward after processing.
type ThinkResponse struct {
	Content     string `json:"content"`
	IrisProfile string `json:"iris_profile"`
	Blocked     bool   `json:"blocked"`
	BlockReason string `json:"block_reason,omitempty"`
}

// HermeticRequest is evaluated by Obatala (Lisp) for ethical permissibility.
type HermeticRequest struct {
	Intent  string          `json:"intent"`
	Action  string          `json:"action"`
	AgentID string          `json:"agent_id"`
	Emotion EmotionSnapshot `json:"emotion"`
}

// HermeticResponse contains Obatala's ruling on the requested action.
type HermeticResponse struct {
	Allowed  bool               `json:"allowed"`
	Decision string             `json:"decision"`
	Scores   map[string]float64 `json:"scores"`
	Overall  float64            `json:"overall"`
}

// MemoryReconstructRequest asks Osun (Julia) to reconstruct relevant memory context.
type MemoryReconstructRequest struct {
	AgentID string          `json:"agent_id"`
	Query   string          `json:"query"`
	Emotion EmotionSnapshot `json:"emotion"`
}

// MemoryReconstructResponse holds the reconstructed context from the SOMA service.
type MemoryReconstructResponse struct {
	Context    string `json:"context"`
	LPMSummary string `json:"lpm_summary"`
}

// RouteThink applies flow enforcement then forwards the think primitive to the
// Rust steward service and returns its response.
func (b *OrishaBridge) RouteThink(ctx context.Context, req ThinkRequest) (ThinkResponse, error) {
	var resp ThinkResponse
	url := b.config.StewardURL + "/think"
	if err := b.post(ctx, url, req, &resp); err != nil {
		return ThinkResponse{}, fmt.Errorf("bridge.RouteThink: %w", err)
	}
	return resp, nil
}

// RouteHermetic forwards an ethics evaluation request to the Obatala (Lisp)
// hermetic service and returns its ruling.
func (b *OrishaBridge) RouteHermetic(ctx context.Context, req HermeticRequest) (HermeticResponse, error) {
	var resp HermeticResponse
	url := b.config.ObatalaURL + "/hermetic/eval"
	if err := b.post(ctx, url, req, &resp); err != nil {
		return HermeticResponse{}, fmt.Errorf("bridge.RouteHermetic: %w", err)
	}
	return resp, nil
}

// RouteMemoryReconstruct forwards a memory reconstruction request to the Osun
// (Julia) SOMA service and returns the reconstructed context.
func (b *OrishaBridge) RouteMemoryReconstruct(ctx context.Context, req MemoryReconstructRequest) (MemoryReconstructResponse, error) {
	var resp MemoryReconstructResponse
	url := b.config.OsunURL + "/soma/reconstruct"
	if err := b.post(ctx, url, req, &resp); err != nil {
		return MemoryReconstructResponse{}, fmt.Errorf("bridge.RouteMemoryReconstruct: %w", err)
	}
	return resp, nil
}

// post marshals body as JSON, POSTs it to url, and unmarshals the response into dst.
// Returns an error if marshaling fails, the request cannot be created or sent, or
// the server responds with a non-2xx status code.
func (b *OrishaBridge) post(ctx context.Context, url string, body any, dst any) error {
	data, err := json.Marshal(body)
	if err != nil {
		return fmt.Errorf("marshal request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := b.client.Do(req)
	if err != nil {
		return fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("upstream returned status %d", resp.StatusCode)
	}

	if err := json.NewDecoder(resp.Body).Decode(dst); err != nil {
		return fmt.Errorf("decode response: %w", err)
	}
	return nil
}
