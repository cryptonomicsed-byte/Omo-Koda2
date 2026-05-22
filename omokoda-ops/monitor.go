package main

import (
	"sync"
	"time"
)

// NodeStatus represents the status of a node
type NodeStatus struct {
	NodeID          string
	ActiveAgents    int64
	TaskQueue       int64
	MemoryPercent   float64
	CPUPercent      float64
	UptimeSeconds   int64
	LastHealthCheck time.Time
}

// AgentInfo represents information about an agent
type AgentInfo struct {
	ID              string
	Status          string // idle, busy, error
	TasksCompleted  int64
	FailureCount    int64
	LastActiveTime  time.Time
	ReputationScore float64
}

var (
	nodeID          = "node-primary"
	activeAgents    int64
	taskQueueLength int64

	agentsMutex sync.RWMutex
	agents      = make(map[string]*AgentInfo)

	nodeStatusMutex sync.RWMutex
	lastNodeStatus  NodeStatus
)

func init() {
	lastNodeStatus = NodeStatus{
		NodeID:          nodeID,
		LastHealthCheck: time.Now(),
	}

	// Start monitoring loop
	go nodeMonitoringLoop()
}

// GetNodeStatus returns the current node status
func GetNodeStatus() NodeStatus {
	nodeStatusMutex.RLock()
	defer nodeStatusMutex.RUnlock()

	status := lastNodeStatus
	status.UptimeSeconds = int64(time.Since(startTime).Seconds())
	status.LastHealthCheck = time.Now()
	return status
}

// UpdateNodeStatus updates node status
func UpdateNodeStatus(activeAgents, taskQueue int64, memPercent, cpuPercent float64) {
	nodeStatusMutex.Lock()
	defer nodeStatusMutex.Unlock()

	lastNodeStatus = NodeStatus{
		NodeID:          nodeID,
		ActiveAgents:    activeAgents,
		TaskQueue:       taskQueue,
		MemoryPercent:   memPercent,
		CPUPercent:      cpuPercent,
		LastHealthCheck: time.Now(),
	}
}

// GetActiveAgents returns the number of active agents
func GetActiveAgents() int64 {
	agentsMutex.RLock()
	defer agentsMutex.RUnlock()
	return int64(len(agents))
}

// GetTaskQueueLength returns the current task queue length
func GetTaskQueueLength() int64 {
	nodeStatusMutex.RLock()
	defer nodeStatusMutex.RUnlock()
	return lastNodeStatus.TaskQueue
}

// RegisterAgent registers a new agent in the monitor
func RegisterAgent(id string, reputationScore float64) {
	agentsMutex.Lock()
	defer agentsMutex.Unlock()

	agents[id] = &AgentInfo{
		ID:              id,
		Status:          "idle",
		TasksCompleted:  0,
		FailureCount:    0,
		LastActiveTime:  time.Now(),
		ReputationScore: reputationScore,
	}

	IncrementAgentCreated()
}

// UnregisterAgent removes an agent from the monitor
func UnregisterAgent(id string) {
	agentsMutex.Lock()
	defer agentsMutex.Unlock()

	delete(agents, id)
	IncrementAgentDestroyed()
}

// UpdateAgentStatus updates an agent's status
func UpdateAgentStatus(id, status string) {
	agentsMutex.Lock()
	defer agentsMutex.Unlock()

	if agent, ok := agents[id]; ok {
		agent.Status = status
		agent.LastActiveTime = time.Now()
	}
}

// RecordAgentTask records that an agent completed a task
func RecordAgentTask(id string, success bool) {
	agentsMutex.Lock()
	defer agentsMutex.Unlock()

	if agent, ok := agents[id]; ok {
		if success {
			agent.TasksCompleted++
			IncrementTasksCompleted()
		} else {
			agent.FailureCount++
		}
		agent.LastActiveTime = time.Now()
	}
}

// GetAgentInfo returns information about a specific agent
func GetAgentInfo(id string) *AgentInfo {
	agentsMutex.RLock()
	defer agentsMutex.RUnlock()

	if agent, ok := agents[id]; ok {
		// Return a copy
		info := *agent
		return &info
	}
	return nil
}

// GetAllAgents returns all agents
func GetAllAgents() map[string]*AgentInfo {
	agentsMutex.RLock()
	defer agentsMutex.RUnlock()

	// Return a copy
	result := make(map[string]*AgentInfo)
	for k, v := range agents {
		agent := *v
		result[k] = &agent
	}
	return result
}

func nodeMonitoringLoop() {
	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		// Update node status with current metrics
		activeCount := GetActiveAgents()
		taskQueue := GetTaskQueueLength()
		UpdateNodeStatus(activeCount, taskQueue, getMemoryPercent(), getCPUUsagePercent())

		// Check for stale agents (not active for 5 minutes)
		checkStaleAgents()
	}
}

func checkStaleAgents() {
	agentsMutex.Lock()
	defer agentsMutex.Unlock()

	now := time.Now()
	staleThreshold := 5 * time.Minute

	for _, agent := range agents {
		if now.Sub(agent.LastActiveTime) > staleThreshold {
			// Mark as stale - in a real system, might trigger remediation
			agent.Status = "stale"
		}
	}
}
