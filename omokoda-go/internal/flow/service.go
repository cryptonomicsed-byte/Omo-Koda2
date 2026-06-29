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

func (s *FlowService) EnforceFlow(agentID string, tier int) error {
	if isSabbath() {
		return fmt.Errorf("rhythm_constraint: Saturday 00:00-01:00 UTC — Sabbath gate active, no actions allowed")
	}
	if !s.limiter.Allow(agentID, tier) {
		return fmt.Errorf("rate_limit_exceeded: tier %d limit reached for agent %s", tier, agentID)
	}
	return nil
}

func isSabbath() bool {
	now := time.Now().UTC()
	return now.Weekday() == time.Saturday && now.Hour() == 0
}

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
