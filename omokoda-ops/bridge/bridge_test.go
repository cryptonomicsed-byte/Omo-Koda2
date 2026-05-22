package bridge

import (
	"context"
	"testing"
	"time"
)

func TestNewBridge(t *testing.T) {
	b := New("test-bridge")
	if b.ID() != "test-bridge" {
		t.Errorf("expected ID test-bridge, got %s", b.ID())
	}
	if b.Status() != StatusDisconnected {
		t.Errorf("expected disconnected, got %s", b.Status())
	}
}

func TestConnectDisconnect(t *testing.T) {
	b := New("b1")
	b.Connect()
	if b.Status() != StatusConnected {
		t.Errorf("expected connected")
	}
	b.Disconnect()
	if b.Status() != StatusDisconnected {
		t.Errorf("expected disconnected")
	}
}

func TestSendRequiresConnected(t *testing.T) {
	b := New("b1")
	msg, _ := NewMessage(TypeEvent, "test", "hello")
	if err := b.Send(msg); err == nil {
		t.Error("expected error sending on disconnected bridge")
	}
}

func TestSendAndReceive(t *testing.T) {
	b := New("b1")
	b.Connect()

	msg, err := NewMessage(TypeEvent, "tasks", map[string]string{"action": "run"})
	if err != nil {
		t.Fatal(err)
	}

	if err := b.Send(msg); err != nil {
		t.Fatal(err)
	}

	select {
	case got := <-b.Outbound():
		if got.Channel != "tasks" {
			t.Errorf("expected channel tasks, got %s", got.Channel)
		}
	case <-time.After(time.Second):
		t.Error("timeout waiting for message")
	}
}

func TestDeliver(t *testing.T) {
	b := New("b1")
	msg, _ := NewMessage(TypeRequest, "permissions", "check")
	b.Deliver(msg)

	select {
	case got := <-b.Inbound():
		if got.Channel != "permissions" {
			t.Errorf("expected channel permissions, got %s", got.Channel)
		}
	case <-time.After(time.Second):
		t.Error("timeout waiting for delivered message")
	}
}

func TestPermissionRequestResponse(t *testing.T) {
	b := New("b1")
	b.Connect()

	req := PermissionRequest{
		RequestID: "req-1",
		AgentID:   "agent-42",
		Tool:      "bash",
		Params:    "ls -la",
		RiskLevel: "low",
	}

	go func() {
		// Simulate remote side draining outbound and sending response
		<-b.Outbound()
		b.HandlePermissionResponse(PermissionResponse{
			RequestID: "req-1",
			Granted:   true,
		})
	}()

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	resp, err := b.RequestPermission(ctx, req)
	if err != nil {
		t.Fatal(err)
	}
	if !resp.Granted {
		t.Error("expected permission granted")
	}
}

func TestPermissionRequestTimeout(t *testing.T) {
	b := New("b1")
	b.Connect()

	req := PermissionRequest{RequestID: "req-timeout", AgentID: "a", Tool: "bash", Params: "", RiskLevel: "high"}
	ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
	defer cancel()

	_, err := b.RequestPermission(ctx, req)
	if err == nil {
		t.Error("expected timeout error")
	}
}

func TestCloseCancelsPendingPermissions(t *testing.T) {
	b := New("b1")
	b.Connect()

	req := PermissionRequest{RequestID: "req-close", AgentID: "a", Tool: "exec", Params: "", RiskLevel: "high"}

	done := make(chan error, 1)
	go func() {
		ctx := context.Background()
		_, err := b.RequestPermission(ctx, req)
		done <- err
	}()

	// Drain outbound so Send doesn't block
	go func() { <-b.Outbound() }()

	time.Sleep(20 * time.Millisecond)
	b.Close()

	select {
	case err := <-done:
		if err == nil {
			t.Error("expected error after bridge close")
		}
	case <-time.After(time.Second):
		t.Error("timeout waiting for close to unblock RequestPermission")
	}
}

func TestStatusSummary(t *testing.T) {
	b := New("b1")
	b.Connect()
	s := b.Summary()
	if s.ID != "b1" {
		t.Errorf("expected ID b1, got %s", s.ID)
	}
	if s.Status != StatusConnected {
		t.Errorf("expected connected")
	}
}
