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

	hub = NewSSEHub(getStewardURL() + "/v1/events")
	go hub.Run()

<<<<<<< HEAD
=======
	// Mesh layer: peer gossip + resource registry + health probes.
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
	agentID := os.Getenv("AGENT_ID")
	if agentID == "" {
		agentID = "omokoda-ops"
	}
	blockID := os.Getenv("MESH_BLOCK_ID")
	if blockID == "" {
		blockID = "default"
	}
<<<<<<< HEAD
	selfAddr := os.Getenv("SELF_ADDR")
=======
	selfAddr := os.Getenv("SELF_ADDR") // e.g. http://my-host:8080
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x

	meshStore := mesh.NewPeerStore()
	gossiper := mesh.NewGossiper(agentID, blockID, selfAddr, meshStore)
	meshHandler := mesh.NewHandler(meshStore, gossiper)

	meshCtx, meshCancel := context.WithCancel(context.Background())
	defer meshCancel()
	go gossiper.Run(meshCtx)

	mux := http.NewServeMux()

	mux.HandleFunc("/health", healthHandler)
	mux.HandleFunc("/ready", readyHandler)
	mux.HandleFunc("/status", statusHandler)
	mux.Handle("/metrics", promhttp.Handler())

	mux.HandleFunc("/v1/birth", proxyToSteward)
	mux.HandleFunc("/v1/think", proxyToSteward)

	mux.HandleFunc("/v1/act", actHandler)

	mux.HandleFunc("/v1/status", proxyToSteward)
	mux.HandleFunc("/v1/health", proxyToSteward)

	mux.Handle("/v1/events", hub)

	mux.HandleFunc("/v1/devices", handleDevices)
	mux.HandleFunc("/v1/devices/", handleDevices)

<<<<<<< HEAD
	meshHandler.RegisterRoutes(mux)

=======
	// --- BlockMesh: gossip, peers, resource registry, health ---
	meshHandler.RegisterRoutes(mux)

	// --- Rhythm gate endpoints (for HttpOyaClient) ---
>>>>>>> origin/claude/omokoda-integration-roadmap-6q0j4x
	mux.HandleFunc("/rhythm/cooldown", rhythmCooldownHandler)
	mux.HandleFunc("/rhythm/record", rhythmRecordHandler)

	server := &http.Server{
		Addr:         ":8080",
		Handler:      mux,
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 0,
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
