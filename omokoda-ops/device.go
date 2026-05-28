package main

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"net/http"
	"strings"
	"sync"
	"time"
)

// DeviceType classifies the physical DePIN hardware.
type DeviceType string

const (
	DeviceHotspot     DeviceType = "hotspot"      // WiFi / LoRa
	DeviceSensor      DeviceType = "sensor"        // temperature, humidity, air quality …
	DeviceComputeNode DeviceType = "compute_node"  // GPU / CPU
	DeviceEnergyMeter DeviceType = "energy_meter"  // solar, grid metering
	DeviceCamera      DeviceType = "camera"        // dashcam, security
	DeviceGeneric     DeviceType = "generic"
)

// DeviceStatus reflects the last known state of the device.
type DeviceStatus string

const (
	DeviceActive   DeviceStatus = "active"
	DeviceInactive DeviceStatus = "inactive"
	DeviceError    DeviceStatus = "error"
)

// GeoPoint holds a WGS-84 coordinate pair.
type GeoPoint struct {
	Lat float64 `json:"lat"`
	Lon float64 `json:"lon"`
}

// Device is the canonical record for a DePIN node.
type Device struct {
	ID           string            `json:"id"`
	Name         string            `json:"name"`
	Type         DeviceType        `json:"type"`
	Owner        string            `json:"owner"` // agent ID or wallet address
	Location     *GeoPoint         `json:"location,omitempty"`
	Status       DeviceStatus      `json:"status"`
	Tier         uint8             `json:"tier"`
	RegisteredAt time.Time         `json:"registered_at"`
	LastSeen     time.Time         `json:"last_seen"`
	Metadata     map[string]string `json:"metadata,omitempty"`
}

// RegisterDeviceRequest is the payload for POST /v1/devices.
type RegisterDeviceRequest struct {
	Name     string            `json:"name"`
	Type     DeviceType        `json:"type"`
	Owner    string            `json:"owner"`
	Location *GeoPoint         `json:"location,omitempty"`
	Metadata map[string]string `json:"metadata,omitempty"`
}

var registry = struct {
	mu      sync.RWMutex
	devices map[string]*Device
}{
	devices: make(map[string]*Device),
}

func newDeviceID() string {
	b := make([]byte, 8)
	rand.Read(b) //nolint:errcheck
	return "dev-" + hex.EncodeToString(b)
}

// handleDevices routes /v1/devices and /v1/devices/{id} to the appropriate
// sub-handler based on method and whether an ID is present.
func handleDevices(w http.ResponseWriter, r *http.Request) {
	// Strip the prefix and any trailing slash.
	id := strings.TrimPrefix(r.URL.Path, "/v1/devices")
	id = strings.Trim(id, "/")

	switch {
	case id == "" && r.Method == http.MethodGet:
		listDevices(w)
	case id == "" && r.Method == http.MethodPost:
		registerDevice(w, r)
	case id != "" && r.Method == http.MethodGet:
		getDevice(w, id)
	case id != "" && r.Method == http.MethodPatch:
		heartbeatDevice(w, r, id)
	default:
		jsonError(w, "method not allowed", http.StatusMethodNotAllowed)
	}
}

func listDevices(w http.ResponseWriter) {
	registry.mu.RLock()
	list := make([]*Device, 0, len(registry.devices))
	for _, d := range registry.devices {
		cp := *d
		list = append(list, &cp)
	}
	registry.mu.RUnlock()

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]any{ //nolint:errcheck
		"devices": list,
		"count":   len(list),
	})
}

func registerDevice(w http.ResponseWriter, r *http.Request) {
	var req RegisterDeviceRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		jsonError(w, "invalid JSON", http.StatusBadRequest)
		return
	}
	if req.Name == "" || req.Type == "" {
		jsonError(w, "name and type are required", http.StatusBadRequest)
		return
	}

	now := time.Now().UTC()
	d := &Device{
		ID:           newDeviceID(),
		Name:         req.Name,
		Type:         req.Type,
		Owner:        req.Owner,
		Location:     req.Location,
		Status:       DeviceActive,
		Tier:         0,
		RegisteredAt: now,
		LastSeen:     now,
		Metadata:     req.Metadata,
	}

	registry.mu.Lock()
	registry.devices[d.ID] = d
	registry.mu.Unlock()

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(d) //nolint:errcheck
}

func getDevice(w http.ResponseWriter, id string) {
	registry.mu.RLock()
	d, ok := registry.devices[id]
	var cp Device
	if ok {
		cp = *d
	}
	registry.mu.RUnlock()

	if !ok {
		jsonError(w, "device not found", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(cp) //nolint:errcheck
}

// heartbeatDevice handles PATCH /v1/devices/{id} — devices call this to
// report they are alive and optionally update their status.
func heartbeatDevice(w http.ResponseWriter, r *http.Request, id string) {
	var req struct {
		Status DeviceStatus `json:"status"`
	}
	json.NewDecoder(r.Body).Decode(&req) //nolint:errcheck

	registry.mu.Lock()
	d, ok := registry.devices[id]
	if ok {
		if req.Status != "" {
			d.Status = req.Status
		}
		d.LastSeen = time.Now().UTC()
	}
	registry.mu.Unlock()

	if !ok {
		jsonError(w, "device not found", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"}) //nolint:errcheck
}
