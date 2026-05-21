package main

import (
	"sync"
	"time"
)

// HealthStatus represents the health of the service
type HealthStatus struct {
	Healthy       bool
	UptimeSeconds int64
	LastCheck     time.Time
}

var (
	startTime     time.Time
	healthMutex   sync.RWMutex
	lastHealth    HealthStatus
	checkInterval = 5 * time.Second
)

func init() {
	startTime = time.Now()
	lastHealth = HealthStatus{
		Healthy:   true,
		LastCheck: startTime,
	}

	// Start health check loop
	go healthCheckLoop()
}

// GetHealth returns the current health status
func GetHealth() HealthStatus {
	healthMutex.RLock()
	defer healthMutex.RUnlock()

	health := lastHealth
	health.UptimeSeconds = int64(time.Since(startTime).Seconds())
	return health
}

// SetHealth updates the health status
func SetHealth(healthy bool) {
	healthMutex.Lock()
	defer healthMutex.Unlock()

	lastHealth = HealthStatus{
		Healthy:   healthy,
		LastCheck: time.Now(),
	}
}

// IsReady checks if the service is ready to accept requests
func IsReady() bool {
	// In a real implementation, this would check if all dependencies
	// (database, cache, message queue, etc.) are available
	health := GetHealth()
	return health.Healthy
}

func healthCheckLoop() {
	ticker := time.NewTicker(checkInterval)
	defer ticker.Stop()

	for range ticker.C {
		// Perform health checks
		healthy := performHealthChecks()
		SetHealth(healthy)
	}
}

func performHealthChecks() bool {
	// Check system resources
	memPercent := getMemoryPercent()
	if memPercent > 90.0 {
		return false
	}

	// Check active connections
	activeAgents := GetActiveAgents()
	if activeAgents < 0 {
		return false
	}

	return true
}

// Helper function to get memory usage percentage
func getMemoryPercent() float64 {
	// In a real implementation, this would use /proc/meminfo or similar
	// For now, return a simulated value
	return 45.0
}
