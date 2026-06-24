package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/omo-koda/omokoda-ops/mesh"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var hub *SSEHub

func main() {
	fmt.Println("Starting Ọmọ Kọ́dà Operations Service")

	InitializeMetrics()

	// Start SSE relay: connects to Rust Steward and fans out to downstream clients.
	hub = NewSSEHub(getStewardURL() + "/v1/events")
	go hub.Run()

	// Mesh layer: peer gossip + resource registry + health probes.
	agentID := os.Getenv("AGENT_ID")
	if agentID == "" {
		agentID = "omokoda-ops"
	}
	blockID := os.Getenv("MESH_BLOCK_ID")
	if blockID == "" {
		blockID = "default"
	}
	selfAddr := os.Getenv("SELF_ADDR") // e.g. http://my-host:8080

	meshStore := mesh.NewPeerStore()
	gossiper := mesh.NewGossiper(agentID, blockID, selfAddr, meshStore)
	meshHandler := mesh.NewHandler(meshStore, gossiper)

	meshCtx, meshCancel := context.WithCancel(context.Background())
	defer meshCancel()
	go gossiper.Run(meshCtx)

	mux := http.NewServeMux()

	// --- Operations endpoints (health, metrics) ---
	mux.HandleFunc("/health", healthHandler)
	mux.HandleFunc("/ready", readyHandler)
	mux.HandleFunc("/status", statusHandler)
	mux.Handle("/metrics", promhttp.Handler())

	// --- Steward gateway: birth + think (direct proxy) ---
	mux.HandleFunc("/v1/birth", proxyToSteward)
	mux.HandleFunc("/v1/think", proxyToSteward)

	// --- Steward gateway: act (rhythm-gated proxy) ---
	mux.HandleFunc("/v1/act", actHandler)

	// --- Steward gateway: read-only pass-throughs ---
	mux.HandleFunc("/v1/status", proxyToSteward)
	mux.HandleFunc("/v1/health", proxyToSteward)

	// --- SSE fan-out (hub proxies from Rust, fans out to N clients) ---
	mux.Handle("/v1/events", hub)

	// --- DePIN device registry ---
	mux.HandleFunc("/v1/devices", handleDevices)
	mux.HandleFunc("/v1/devices/", handleDevices)

	// --- BlockMesh: gossip, peers, resource registry, health ---
	meshHandler.RegisterRoutes(mux)

	server := &http.Server{
		Addr:         ":8080",
		Handler:      mux,
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 0, // 0 = no timeout; required for SSE long-lived connections
		IdleTimeout:  120 * time.Second,
	}

	go func() {
		log.Printf("Server listening on %s (steward at %s)", server.Addr, getStewardURL())
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server error: %v", err)
		}
	}()

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	fmt.Println("\nShutting down server...")
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		log.Fatalf("Server shutdown error: %v", err)
	}
	fmt.Println("Server stopped")
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	health := GetHealth()
	w.Header().Set("Content-Type", "application/json")
	if !health.Healthy {
		w.WriteHeader(http.StatusServiceUnavailable)
	}
	fmt.Fprintf(w, `{"healthy":%v,"uptime":%d}`, health.Healthy, health.UptimeSeconds)
}

func readyHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	if !IsReady() {
		w.WriteHeader(http.StatusServiceUnavailable)
		fmt.Fprint(w, `{"ready":false}`)
		return
	}
	fmt.Fprint(w, `{"ready":true}`)
}

func statusHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	status := GetNodeStatus()
	w.Header().Set("Content-Type", "application/json")
	fmt.Fprintf(w, `{"node_id":"%s","active_agents":%d,"task_queue":%d,"memory_percent":%.2f}`,
		status.NodeID, status.ActiveAgents, status.TaskQueue, status.MemoryPercent)
}
