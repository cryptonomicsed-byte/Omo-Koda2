package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

// TestRhythmAllow verifies non-rate-limited tools pass immediately.
func TestRhythmAllow(t *testing.T) {
	d := CheckRhythm("read_file", "agent-1")
	if d != RhythmAllow {
		t.Errorf("expected Allow, got %v", d)
	}
}

// TestRhythmCooldown verifies repeated bash calls are rate-limited.
func TestRhythmCooldown(t *testing.T) {
	// Reset tracker for this test.
	tracker.mu.Lock()
	tracker.lastUsed = make(map[string]time.Time)
	tracker.mu.Unlock()

	first := CheckRhythm("bash", "agent-cd")
	if first != RhythmAllow {
		t.Errorf("expected first bash call to be Allow, got %v", first)
	}
	second := CheckRhythm("bash", "agent-cd")
	if second != RhythmCooldown {
		t.Errorf("expected second bash call to be Cooldown, got %v", second)
	}
}

// TestRhythmCooldownPerAgent verifies cooldowns are per-agent, not global.
func TestRhythmCooldownPerAgent(t *testing.T) {
	tracker.mu.Lock()
	tracker.lastUsed = make(map[string]time.Time)
	tracker.mu.Unlock()

	CheckRhythm("bash", "agent-a")
	d := CheckRhythm("bash", "agent-b")
	if d != RhythmAllow {
		t.Errorf("expected different agent to be allowed; got %v", d)
	}
}

// TestIsSabbath just verifies the function runs without panic.
func TestIsSabbath(t *testing.T) {
	_ = IsSabbath()
}

// TestRegisterDeviceEndpoint verifies POST /v1/devices creates a device.
func TestRegisterDeviceEndpoint(t *testing.T) {
	// Clear registry.
	registry.mu.Lock()
	registry.devices = make(map[string]*Device)
	registry.mu.Unlock()

	body, _ := json.Marshal(RegisterDeviceRequest{
		Name:  "Test Hotspot",
		Type:  DeviceHotspot,
		Owner: "agent-abc",
	})

	req := httptest.NewRequest(http.MethodPost, "/v1/devices", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	handleDevices(w, req)

	if w.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", w.Code, w.Body.String())
	}

	var dev Device
	if err := json.NewDecoder(w.Body).Decode(&dev); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if dev.ID == "" {
		t.Error("expected device ID to be set")
	}
	if dev.Name != "Test Hotspot" {
		t.Errorf("expected name 'Test Hotspot', got %q", dev.Name)
	}
	if dev.Status != DeviceActive {
		t.Errorf("expected status active, got %q", dev.Status)
	}
}

// TestListDevicesEndpoint verifies GET /v1/devices returns the registry.
func TestListDevicesEndpoint(t *testing.T) {
	registry.mu.Lock()
	registry.devices = map[string]*Device{
		"dev-1": {ID: "dev-1", Name: "Sensor A", Type: DeviceSensor, Status: DeviceActive},
		"dev-2": {ID: "dev-2", Name: "Camera B", Type: DeviceCamera, Status: DeviceActive},
	}
	registry.mu.Unlock()

	req := httptest.NewRequest(http.MethodGet, "/v1/devices", nil)
	w := httptest.NewRecorder()
	handleDevices(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}

	var resp struct {
		Count int `json:"count"`
	}
	json.NewDecoder(w.Body).Decode(&resp) //nolint:errcheck
	if resp.Count != 2 {
		t.Errorf("expected count 2, got %d", resp.Count)
	}
}

// TestGetDeviceEndpoint verifies GET /v1/devices/{id} retrieves a single device.
func TestGetDeviceEndpoint(t *testing.T) {
	registry.mu.Lock()
	registry.devices = map[string]*Device{
		"dev-xyz": {ID: "dev-xyz", Name: "Node Alpha", Type: DeviceComputeNode, Status: DeviceActive},
	}
	registry.mu.Unlock()

	req := httptest.NewRequest(http.MethodGet, "/v1/devices/dev-xyz", nil)
	w := httptest.NewRecorder()
	handleDevices(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", w.Code, w.Body.String())
	}
	var dev Device
	json.NewDecoder(w.Body).Decode(&dev) //nolint:errcheck
	if dev.ID != "dev-xyz" {
		t.Errorf("expected id dev-xyz, got %q", dev.ID)
	}
}

// TestGetDeviceNotFound verifies 404 for unknown device IDs.
func TestGetDeviceNotFound(t *testing.T) {
	registry.mu.Lock()
	registry.devices = make(map[string]*Device)
	registry.mu.Unlock()

	req := httptest.NewRequest(http.MethodGet, "/v1/devices/does-not-exist", nil)
	w := httptest.NewRecorder()
	handleDevices(w, req)

	if w.Code != http.StatusNotFound {
		t.Errorf("expected 404, got %d", w.Code)
	}
}

// TestHeartbeatDevice verifies PATCH /v1/devices/{id} updates LastSeen.
func TestHeartbeatDevice(t *testing.T) {
	before := time.Now().UTC().Add(-1 * time.Minute)
	registry.mu.Lock()
	registry.devices = map[string]*Device{
		"dev-hb": {ID: "dev-hb", Name: "Probe", Type: DeviceGeneric, Status: DeviceActive, LastSeen: before},
	}
	registry.mu.Unlock()

	body, _ := json.Marshal(map[string]string{"status": "active"})
	req := httptest.NewRequest(http.MethodPatch, "/v1/devices/dev-hb", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	handleDevices(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", w.Code, w.Body.String())
	}

	registry.mu.RLock()
	updated := registry.devices["dev-hb"].LastSeen
	registry.mu.RUnlock()

	if !updated.After(before) {
		t.Errorf("expected LastSeen to be updated after heartbeat")
	}
}
