package main

import (
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/omo-koda/omokoda-go/internal/flow"
	"github.com/omo-koda/omokoda-go/internal/ratelimit"
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

	limiter := ratelimit.New(0) // zero ttl → defaults to 1 hour idle-bucket expiry
	svc := flow.NewService(limiter)
	store := flow.NewPrimitiveStore()

	lis, err := net.Listen("tcp", ":"+tcpPort)
	if err != nil {
		log.Fatalf("Failed to listen on TCP :%s: %v", tcpPort, err)
	}
	log.Printf("ỌYA flow service listening on TCP :%s", tcpPort)
	go svc.Serve(lis)

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
