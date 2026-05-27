package main

import (
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/omo-koda/oya/internal/flow"
	"github.com/omo-koda/oya/internal/ratelimit"
)

func main() {
	port := os.Getenv("OYA_PORT")
	if port == "" {
		port = "50052"
	}

	limiter := ratelimit.New()
	svc := flow.NewService(limiter)

	lis, err := net.Listen("tcp", ":"+port)
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	log.Printf("ỌYA flow service listening on :%s", port)
	go svc.Serve(lis)

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	log.Println("ỌYA shutting down")
}
