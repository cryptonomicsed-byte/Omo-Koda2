// Command oya is the ỌYA flow-control gRPC server.
//
// It listens for FlowService RPCs on :50052 and exposes Prometheus-compatible
// metrics on :9090/metrics.
//
// Graceful shutdown is triggered by SIGINT or SIGTERM.
//
// gRPC wiring: when google.golang.org/grpc is added to go.mod, register the
// FlowServiceServer with grpc.NewServer() and call RegisterFlowServiceServer.
// The proto-generated stubs live in proto/oya.proto and must be compiled with
// protoc + protoc-gen-go-grpc before importing.  Until then, the server uses a
// raw net.Listener so the binary can be built and tested without a protoc step.
package main

import (
	"context"
	"expvar"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"sync/atomic"
	"syscall"
	"time"

	"github.com/omo-koda/omokoda-go/internal/flow"
)

const (
	grpcAddr    = ":50052"
	metricsAddr = ":9090"
)

// Prometheus-style counters exposed via /metrics (expvar format).
var (
	requestsTotal  = expvar.NewInt("oya_requests_total")
	requestsDenied = expvar.NewInt("oya_requests_denied_total")
	activeStreams   = expvar.NewInt("oya_active_streams")
)

func main() {
	log.SetPrefix("[oya] ")
	log.SetFlags(log.LstdFlags | log.Lshortfile)

	svc := flow.New()
	defer svc.Stop()

	ctx, cancel := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer cancel()

	// --- gRPC listener ---
	grpcLn, err := net.Listen("tcp", grpcAddr)
	if err != nil {
		log.Fatalf("failed to listen on %s: %v", grpcAddr, err)
	}
	log.Printf("gRPC listener ready on %s", grpcAddr)

	// Serve a minimal placeholder until protoc-generated stubs are available.
	// Replace this block with grpc.NewServer() + RegisterFlowServiceServer once
	// google.golang.org/grpc is added to go.mod and protos are compiled.
	var grpcConnCount atomic.Int64
	go func() {
		for {
			conn, err := grpcLn.Accept()
			if err != nil {
				// Listener closed — server is shutting down.
				return
			}
			grpcConnCount.Add(1)
			go func(c net.Conn) {
				defer c.Close()
				defer grpcConnCount.Add(-1)
				// In production this goroutine is replaced by grpc.Server's
				// connection handler.  For now, close immediately so health
				// probes that open a TCP connection still succeed.
			}(conn)
		}
	}()

	// --- Metrics / health HTTP server ---
	mux := http.NewServeMux()

	// /metrics — expvar JSON (drop-in for Prometheus text format adapters).
	mux.Handle("/metrics", expvar.Handler())

	// /healthz — simple liveness probe.
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintln(w, "ok")
	})

	// /readyz — readiness probe (delegates to a sample EnforceFlow dry-run).
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		// We use a synthetic internal agent with T5 (never rate-limited during
		// normal operation) to verify that the service stack is healthy.
		if err := svc.EnforceFlow("__health__", 5); err != nil {
			http.Error(w, "not ready: "+err.Error(), http.StatusServiceUnavailable)
			return
		}
		fmt.Fprintln(w, "ok")
	})

	httpSrv := &http.Server{
		Addr:         metricsAddr,
		Handler:      mux,
		ReadTimeout:  5 * time.Second,
		WriteTimeout: 5 * time.Second,
	}

	go func() {
		log.Printf("metrics server listening on %s/metrics", metricsAddr)
		if err := httpSrv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("metrics server error: %v", err)
		}
	}()

	// --- Wait for shutdown signal ---
	<-ctx.Done()
	log.Println("shutdown signal received — stopping ỌYA")

	// Graceful HTTP shutdown (5 s deadline).
	shutCtx, shutCancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer shutCancel()
	if err := httpSrv.Shutdown(shutCtx); err != nil {
		log.Printf("metrics server shutdown error: %v", err)
	}

	// Close the gRPC listener; running connections drain naturally.
	if err := grpcLn.Close(); err != nil {
		log.Printf("gRPC listener close error: %v", err)
	}

	log.Printf("ỌYA stopped cleanly (gRPC connections served: %d)", grpcConnCount.Load())
	os.Exit(0)
}
