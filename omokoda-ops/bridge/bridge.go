package bridge

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"time"
)

type MessageType string

const (
	TypeRequest  MessageType = "request"
	TypeResponse MessageType = "response"
	TypeEvent    MessageType = "event"
	TypeError    MessageType = "error"
)

type Status string

const (
	StatusDisconnected Status = "disconnected"
	StatusConnecting   Status = "connecting"
	StatusConnected    Status = "connected"
	StatusError        Status = "error"
)

// Message is the fundamental unit of bridge communication.
type Message struct {
	ID          string          `json:"id"`
	Type        MessageType     `json:"type"`
	Channel     string          `json:"channel"` // "permissions", "tasks", "status", "events"
	Payload     json.RawMessage `json:"payload"`
	TimestampMS int64           `json:"timestamp_ms"`
}

func NewMessage(typ MessageType, channel string, payload interface{}) (Message, error) {
	b, err := json.Marshal(payload)
	if err != nil {
		return Message{}, err
	}
	return Message{
		ID:          fmt.Sprintf("%d", time.Now().UnixNano()),
		Type:        typ,
		Channel:     channel,
		Payload:     json.RawMessage(b),
		TimestampMS: time.Now().UnixMilli(),
	}, nil
}

// PermissionRequest is forwarded across the bridge when the agent needs user approval.
type PermissionRequest struct {
	RequestID string `json:"request_id"`
	AgentID   string `json:"agent_id"`
	Tool      string `json:"tool"`
	Params    string `json:"params"`
	RiskLevel string `json:"risk_level"` // "low", "medium", "high"
}

// PermissionResponse is the bridge reply to a PermissionRequest.
type PermissionResponse struct {
	RequestID string `json:"request_id"`
	Granted   bool   `json:"granted"`
	Reason    string `json:"reason,omitempty"`
}

// NodeBridge manages a bidirectional channel between the local node and a remote peer.
type NodeBridge struct {
	id     string
	mu     sync.RWMutex
	status Status

	inbound  chan Message
	outbound chan Message
	done     chan struct{}

	permMu       sync.Mutex
	pendingPerms map[string]chan PermissionResponse
}

func New(id string) *NodeBridge {
	return &NodeBridge{
		id:           id,
		status:       StatusDisconnected,
		inbound:      make(chan Message, 128),
		outbound:     make(chan Message, 128),
		done:         make(chan struct{}),
		pendingPerms: make(map[string]chan PermissionResponse),
	}
}

func (b *NodeBridge) ID() string { return b.id }

func (b *NodeBridge) Connect() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusConnected
}

func (b *NodeBridge) Disconnect() {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.status != StatusDisconnected {
		b.status = StatusDisconnected
	}
}

func (b *NodeBridge) SetStatus(s Status) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = s
}

func (b *NodeBridge) Status() Status {
	b.mu.RLock()
	defer b.mu.RUnlock()
	return b.status
}

// Send enqueues a message for delivery to the remote peer.
func (b *NodeBridge) Send(msg Message) error {
	if b.Status() != StatusConnected {
		return fmt.Errorf("bridge %s is not connected (status: %s)", b.id, b.Status())
	}
	select {
	case b.outbound <- msg:
		return nil
	case <-b.done:
		return fmt.Errorf("bridge closed")
	default:
		return fmt.Errorf("outbound queue full")
	}
}

// Inbound returns the channel of messages arriving from the remote peer.
func (b *NodeBridge) Inbound() <-chan Message { return b.inbound }

// Outbound returns the channel of messages queued for sending.
func (b *NodeBridge) Outbound() <-chan Message { return b.outbound }

// Deliver places a message on the inbound channel (used by transport layer).
func (b *NodeBridge) Deliver(msg Message) {
	select {
	case b.inbound <- msg:
	case <-b.done:
	}
}

// Close shuts down the bridge and cancels pending permission requests.
func (b *NodeBridge) Close() {
	b.mu.Lock()
	b.status = StatusDisconnected
	b.mu.Unlock()

	select {
	case <-b.done:
	default:
		close(b.done)
	}

	b.permMu.Lock()
	for id, ch := range b.pendingPerms {
		close(ch)
		delete(b.pendingPerms, id)
	}
	b.permMu.Unlock()
}

// Done returns a channel closed when the bridge is shut down.
func (b *NodeBridge) Done() <-chan struct{} { return b.done }

// RequestPermission sends a permission request across the bridge and waits for a response.
func (b *NodeBridge) RequestPermission(ctx context.Context, req PermissionRequest) (PermissionResponse, error) {
	ch := make(chan PermissionResponse, 1)

	b.permMu.Lock()
	b.pendingPerms[req.RequestID] = ch
	b.permMu.Unlock()

	defer func() {
		b.permMu.Lock()
		delete(b.pendingPerms, req.RequestID)
		b.permMu.Unlock()
	}()

	msg, err := NewMessage(TypeRequest, "permissions", req)
	if err != nil {
		return PermissionResponse{}, err
	}
	if err := b.Send(msg); err != nil {
		return PermissionResponse{}, err
	}

	select {
	case resp, ok := <-ch:
		if !ok {
			return PermissionResponse{}, fmt.Errorf("bridge closed while waiting for permission")
		}
		return resp, nil
	case <-ctx.Done():
		return PermissionResponse{}, ctx.Err()
	}
}

// HandlePermissionResponse routes a response back to the waiting RequestPermission call.
func (b *NodeBridge) HandlePermissionResponse(resp PermissionResponse) {
	b.permMu.Lock()
	ch, ok := b.pendingPerms[resp.RequestID]
	b.permMu.Unlock()
	if ok {
		select {
		case ch <- resp:
		default:
		}
	}
}

// StatusSummary is a JSON-serialisable bridge snapshot.
type StatusSummary struct {
	ID             string `json:"id"`
	Status         Status `json:"status"`
	InboundQueued  int    `json:"inbound_queued"`
	OutboundQueued int    `json:"outbound_queued"`
	PendingPerms   int    `json:"pending_permissions"`
}

func (b *NodeBridge) Summary() StatusSummary {
	b.permMu.Lock()
	perms := len(b.pendingPerms)
	b.permMu.Unlock()
	return StatusSummary{
		ID:             b.id,
		Status:         b.Status(),
		InboundQueued:  len(b.inbound),
		OutboundQueued: len(b.outbound),
		PendingPerms:   perms,
	}
}
