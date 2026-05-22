package remote

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/omo-koda/omokoda-ops/bridge"
)

func TestConnect(t *testing.T) {
	mgr := NewManager()
	sess, err := mgr.Connect("http://node-2:8080", "agent-1")
	if err != nil {
		t.Fatal(err)
	}
	if sess.Status() != SessionConnected {
		t.Errorf("expected connected, got %s", sess.Status())
	}
	if mgr.Count() != 1 {
		t.Errorf("expected 1 session, got %d", mgr.Count())
	}
}

func TestConnectValidation(t *testing.T) {
	mgr := NewManager()
	if _, err := mgr.Connect("", "agent-1"); err == nil {
		t.Error("expected error for empty nodeURL")
	}
	if _, err := mgr.Connect("http://node:8080", ""); err == nil {
		t.Error("expected error for empty agentID")
	}
}

func TestDisconnect(t *testing.T) {
	mgr := NewManager()
	sess, _ := mgr.Connect("http://node:8080", "a1")
	if err := mgr.Disconnect(sess.ID); err != nil {
		t.Fatal(err)
	}
	if mgr.Count() != 0 {
		t.Errorf("expected 0 sessions after disconnect")
	}
	if sess.Status() != SessionDisconnected {
		t.Errorf("expected disconnected status")
	}
}

func TestDisconnectUnknown(t *testing.T) {
	mgr := NewManager()
	if err := mgr.Disconnect("no-such-session"); err == nil {
		t.Error("expected error disconnecting unknown session")
	}
}

func TestGetSession(t *testing.T) {
	mgr := NewManager()
	sess, _ := mgr.Connect("http://node:8080", "a1")
	got, ok := mgr.Get(sess.ID)
	if !ok || got.ID != sess.ID {
		t.Error("Get did not return the expected session")
	}
}

func TestListSessions(t *testing.T) {
	mgr := NewManager()
	mgr.Connect("http://node-1:8080", "a1")
	mgr.Connect("http://node-2:8080", "a2")
	if len(mgr.List()) != 2 {
		t.Errorf("expected 2 sessions, got %d", len(mgr.List()))
	}
}

func TestPermissionBridge(t *testing.T) {
	mgr := NewManager()
	var capturedReq bridge.PermissionRequest
	mgr.SetPermissionBridge(func(sess *RemoteSession, req bridge.PermissionRequest) bridge.PermissionResponse {
		capturedReq = req
		return bridge.PermissionResponse{RequestID: req.RequestID, Granted: true}
	})

	sess, _ := mgr.Connect("http://node:8080", "a1")

	// Simulate a permission request arriving on the inbound channel
	req := bridge.PermissionRequest{RequestID: "r1", AgentID: "a1", Tool: "bash", Params: "ls", RiskLevel: "low"}
	payload, _ := json.Marshal(req)
	sess.bridge.Deliver(bridge.Message{
		ID:      "m1",
		Type:    bridge.TypeRequest,
		Channel: "permissions",
		Payload: json.RawMessage(payload),
	})

	time.Sleep(50 * time.Millisecond) // let dispatcher goroutine run

	if capturedReq.RequestID != "r1" {
		t.Errorf("expected request r1, got %q", capturedReq.RequestID)
	}
}

func TestSDKAdapter(t *testing.T) {
	mgr := NewManager()
	sess, _ := mgr.Connect("http://node:8080", "a1")
	adapter := NewSDKAdapter(sess)

	raw := []byte(`{"type":"request","content":{"tool":"grep"}}`)
	msg, err := adapter.Adapt(raw)
	if err != nil {
		t.Fatal(err)
	}
	if msg.Type != "request" {
		t.Errorf("expected type request, got %s", msg.Type)
	}

	back, err := adapter.AdaptResponse(msg)
	if err != nil {
		t.Fatal(err)
	}
	var out SDKMessage
	if err := json.Unmarshal(back, &out); err != nil {
		t.Fatal(err)
	}
	if out.Type != "request" {
		t.Errorf("expected round-trip type request")
	}
}

func TestSessionPing(t *testing.T) {
	mgr := NewManager()
	sess, _ := mgr.Connect("http://node:8080", "a1")
	before := sess.LastPing
	time.Sleep(2 * time.Millisecond)
	sess.Ping()
	if !sess.LastPing.After(before) {
		t.Error("LastPing should advance after Ping()")
	}
}

func TestSessionSummary(t *testing.T) {
	mgr := NewManager()
	sess, _ := mgr.Connect("http://node:8080", "a1")
	s := sess.Summary()
	if s.NodeURL != "http://node:8080" {
		t.Errorf("unexpected NodeURL: %s", s.NodeURL)
	}
	if s.Status != SessionConnected {
		t.Errorf("expected connected")
	}
}
