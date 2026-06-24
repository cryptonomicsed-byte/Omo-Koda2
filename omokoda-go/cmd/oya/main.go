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
	tcpPort := os.Getenv("OYA_PORT")
	if tcpPort == "" {
		tcpPort = "50052"
	}
	httpPort := os.Getenv("OYA_HTTP_PORT")
	if httpPort == "" {
		httpPort = "8080"
	}

	limiter := ratelimit.New()
	svc := flow.NewService(limiter)
	store := flow.NewPrimitiveStore()

	// TCP server (original protocol)
	lis, err := net.Listen("tcp", ":"+tcpPort)
	if err != nil {
		log.Fatalf("Failed to listen on TCP :%s: %v", tcpPort, err)
	}
	log.Printf("ỌYA flow service listening on TCP :%s", tcpPort)
	go svc.Serve(lis)

	// HTTP REST API for OyaClient (Rust) integration
	httpHandler := flow.NewHTTPHandler(store)
	log.Printf("ỌYA HTTP API listening on :%s", httpPort)
	go func() {
		if err := http.ListenAndServe(":"+httpPort, httpHandler); err != nil {
			log.Printf("ỌYA HTTP server error: %v", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	log.Println("ỌYA shutting down")
}
