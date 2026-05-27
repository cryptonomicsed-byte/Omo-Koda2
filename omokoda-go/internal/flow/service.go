package flow

import (
	"fmt"
	"net"
	"time"

	"github.com/omo-koda/oya/internal/ratelimit"
)

// FlowService enforces rate limiting and Sabbath rhythm constraints.
type FlowService struct {
	limiter *ratelimit.Limiter
}

func NewService(limiter *ratelimit.Limiter) *FlowService {
	return &FlowService{limiter: limiter}
}

// EnforceFlow checks rate limit and Sabbath gate for an agent+tier combination.
// Returns nil on allow, or an error message on deny.
func (s *FlowService) EnforceFlow(agentID string, tier int) error {
	if isSabbath() {
		return fmt.Errorf("rhythm_constraint: Sunday 00:00-01:00 UTC — Sabbath gate active, no actions allowed")
	}
	if !s.limiter.Allow(agentID, tier) {
		return fmt.Errorf("rate_limit_exceeded: tier %d limit reached for agent %s", tier, agentID)
	}
	return nil
}

// isSabbath returns true during UTC Sunday 00:00–01:00 (ritual-codex Sabbath enforcement).
func isSabbath() bool {
	now := time.Now().UTC()
	return now.Weekday() == time.Sunday && now.Hour() == 0
}

// Serve listens for simple text commands on a TCP socket (proto stub).
func (s *FlowService) Serve(lis net.Listener) {
	for {
		conn, err := lis.Accept()
		if err != nil {
			return
		}
		go func(c net.Conn) {
			defer c.Close()
			buf := make([]byte, 256)
			n, _ := c.Read(buf)
			agentID := string(buf[:n])
			if err := s.EnforceFlow(agentID, 0); err != nil {
				c.Write([]byte("deny:" + err.Error()))
			} else {
				c.Write([]byte("allow"))
			}
		}(conn)
	}
}
