package remote

import (
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/omo-koda/omokoda-ops/bridge"
)

// SessionStatus is the lifecycle state of a remote session.
type SessionStatus string

const (
	SessionPending      SessionStatus = "pending"
	SessionConnected    SessionStatus = "connected"
	SessionDisconnected SessionStatus = "disconnected"
	SessionError        SessionStatus = "error"
)

// RemoteSession represents an active connection to a remote node.
type RemoteSession struct {
	ID        string
	NodeURL   string
	AgentID   string
	CreatedAt time.Time
	LastPing  time.Time

	mu     sync.RWMutex
	status SessionStatus
	bridge *bridge.NodeBridge
}

func newSession(nodeURL, agentID string, br *bridge.NodeBridge) *RemoteSession {
	return &RemoteSession{
		ID:        fmt.Sprintf("sess-%d", time.Now().UnixNano()),
		NodeURL:   nodeURL,
		AgentID:   agentID,
		CreatedAt: time.Now(),
		LastPing:  time.Now(),
		status:    SessionPending,
		bridge:    br,
	}
}

func (s *RemoteSession) Status() SessionStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

func (s *RemoteSession) SetStatus(st SessionStatus) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = st
}

func (s *RemoteSession) Bridge() *bridge.NodeBridge { return s.bridge }

func (s *RemoteSession) Ping() {
	s.mu.Lock()
	s.LastPing = time.Now()
	s.mu.Unlock()
}

// SessionSummary is a JSON-serialisable session snapshot.
type SessionSummary struct {
	ID        string               `json:"id"`
	NodeURL   string               `json:"node_url"`
	AgentID   string               `json:"agent_id"`
	Status    SessionStatus        `json:"status"`
	CreatedAt time.Time            `json:"created_at"`
	LastPing  time.Time            `json:"last_ping"`
	Bridge    bridge.StatusSummary `json:"bridge"`
}

func (s *RemoteSession) Summary() SessionSummary {
	return SessionSummary{
		ID:        s.ID,
		NodeURL:   s.NodeURL,
		AgentID:   s.AgentID,
		Status:    s.Status(),
		CreatedAt: s.CreatedAt,
		LastPing:  s.LastPing,
		Bridge:    s.bridge.Summary(),
	}
}

// PermissionBridgeFn is called when the remote node forwards a permission request.
type PermissionBridgeFn func(sess *RemoteSession, req bridge.PermissionRequest) bridge.PermissionResponse

// RemoteSessionManager manages active remote sessions with permission bridging.
type RemoteSessionManager struct {
	mu       sync.RWMutex
	sessions map[string]*RemoteSession

	permBridge PermissionBridgeFn
}

func NewManager() *RemoteSessionManager {
	return &RemoteSessionManager{
		sessions: make(map[string]*RemoteSession),
		permBridge: func(_ *RemoteSession, req bridge.PermissionRequest) bridge.PermissionResponse {
			// Default: deny all forwarded permission requests
			return bridge.PermissionResponse{RequestID: req.RequestID, Granted: false, Reason: "no permission bridge configured"}
		},
	}
}

// SetPermissionBridge registers the callback that decides forwarded permission requests.
func (m *RemoteSessionManager) SetPermissionBridge(fn PermissionBridgeFn) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.permBridge = fn
}

// Connect opens a new remote session to nodeURL for agentID.
// The session's bridge is immediately marked Connected.
func (m *RemoteSessionManager) Connect(nodeURL, agentID string) (*RemoteSession, error) {
	if nodeURL == "" {
		return nil, fmt.Errorf("nodeURL must not be empty")
	}
	if agentID == "" {
		return nil, fmt.Errorf("agentID must not be empty")
	}

	br := bridge.New(fmt.Sprintf("bridge-%s-%d", agentID, time.Now().UnixNano()))
	br.Connect()

	sess := newSession(nodeURL, agentID, br)
	sess.SetStatus(SessionConnected)

	// Dispatch permission forwarding for this session's bridge inbound
	go m.dispatchInbound(sess)

	m.mu.Lock()
	m.sessions[sess.ID] = sess
	m.mu.Unlock()

	return sess, nil
}

// dispatchInbound reads inbound bridge messages and handles permission requests.
func (m *RemoteSessionManager) dispatchInbound(sess *RemoteSession) {
	for {
		select {
		case msg, ok := <-sess.bridge.Inbound():
			if !ok {
				return
			}
			if msg.Channel == "permissions" && msg.Type == "request" {
				var req bridge.PermissionRequest
				if err := json.Unmarshal(msg.Payload, &req); err == nil {
					m.mu.RLock()
					fn := m.permBridge
					m.mu.RUnlock()
					resp := fn(sess, req)
					sess.bridge.HandlePermissionResponse(resp)
				}
			}
		case <-sess.bridge.Done():
			return
		}
	}
}

// Disconnect closes the session and its bridge.
func (m *RemoteSessionManager) Disconnect(sessionID string) error {
	m.mu.Lock()
	sess, ok := m.sessions[sessionID]
	if ok {
		delete(m.sessions, sessionID)
	}
	m.mu.Unlock()

	if !ok {
		return fmt.Errorf("session %s not found", sessionID)
	}
	sess.SetStatus(SessionDisconnected)
	sess.bridge.Close()
	return nil
}

// Get returns a session by ID.
func (m *RemoteSessionManager) Get(sessionID string) (*RemoteSession, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	sess, ok := m.sessions[sessionID]
	return sess, ok
}

// List returns all active sessions.
func (m *RemoteSessionManager) List() []*RemoteSession {
	m.mu.RLock()
	defer m.mu.RUnlock()
	out := make([]*RemoteSession, 0, len(m.sessions))
	for _, s := range m.sessions {
		out = append(out, s)
	}
	return out
}

// Count returns the number of active sessions.
func (m *RemoteSessionManager) Count() int {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return len(m.sessions)
}

// SDKMessage is the wire format used by external SDK clients.
type SDKMessage struct {
	Type    string          `json:"type"`
	Content json.RawMessage `json:"content"`
}

// SDKMessageAdapter translates between SDK wire format and internal bridge messages.
type SDKMessageAdapter struct {
	session *RemoteSession
}

func NewSDKAdapter(sess *RemoteSession) *SDKMessageAdapter {
	return &SDKMessageAdapter{session: sess}
}

// Adapt converts a raw SDK message to a BridgeMessage.
func (a *SDKMessageAdapter) Adapt(raw []byte) (bridge.Message, error) {
	var sdk SDKMessage
	if err := json.Unmarshal(raw, &sdk); err != nil {
		return bridge.Message{}, fmt.Errorf("invalid SDK message: %w", err)
	}
	return bridge.Message{
		ID:          fmt.Sprintf("sdk-%d", time.Now().UnixNano()),
		Type:        bridge.MessageType(sdk.Type),
		Channel:     "sdk",
		Payload:     sdk.Content,
		TimestampMS: time.Now().UnixMilli(),
	}, nil
}

// AdaptResponse converts a BridgeMessage back to SDK wire format.
func (a *SDKMessageAdapter) AdaptResponse(msg bridge.Message) ([]byte, error) {
	sdk := SDKMessage{
		Type:    string(msg.Type),
		Content: msg.Payload,
	}
	return json.Marshal(sdk)
}
