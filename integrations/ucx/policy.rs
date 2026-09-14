//! Agent compute policy — governs how an Omo-Koda2 agent uses UCX compute resources.
//!
//! Encodes: spend limits, allowed workload types, provider preferences,
//! budget alert thresholds, and local vs external preference.
//!
//! Loaded from env vars at startup; can be overridden by agent config.

use serde::{Deserialize, Serialize};

/// Compute budget policy for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePolicy {
    /// Maximum spend per job in USD cents. None = no limit.
    pub max_spend_per_job_cents: Option<u64>,
    /// Maximum total spend over the policy window in USD cents. None = no limit.
    pub max_total_spend_cents: Option<u64>,
    /// Running total spent in this window (cents). Agent increments this.
    pub total_spent_cents: u64,
    /// Fraction of max_total_spend_cents that triggers a budget alert (0.0-1.0).
    pub budget_alert_threshold: f64,
    /// Workload types the agent is allowed to submit.
    pub allowed_workloads: Vec<String>,
    /// If true, prefer native (local machine) providers over external.
    pub prefer_native: bool,
    /// If true, external providers (cloud) are allowed at all.
    pub allow_external: bool,
    /// Max acceptable GPU price in cents/hr. None = no ceiling.
    pub max_gpu_price_cents_per_hr: Option<u64>,
    /// Minimum VRAM required per job in GB. None = no minimum.
    pub min_vram_gb: Option<f64>,
    /// Provider IDs to never use. Empty = no deny list.
    pub deny_providers: Vec<String>,
    /// Provider IDs to prefer over others (ordered by priority).
    pub preferred_providers: Vec<String>,
}

impl Default for ComputePolicy {
    fn default() -> Self {
        Self {
            max_spend_per_job_cents:   None,
            max_total_spend_cents:     None,
            total_spent_cents:         0,
            budget_alert_threshold:    0.80,
            allowed_workloads:         vec![
                "Generic".into(), "Training".into(), "Inference".into(),
                "Simulation".into(), "Rendering".into(), "CiCd".into(),
            ],
            prefer_native:             true,
            allow_external:            true,
            max_gpu_price_cents_per_hr: None,
            min_vram_gb:               None,
            deny_providers:            vec![],
            preferred_providers:       vec![],
        }
    }
}

impl ComputePolicy {
    /// Load policy from environment variables. Falls back to defaults.
    pub fn from_env() -> Self {
        let mut p = Self::default();

        if let Ok(v) = std::env::var("UCX_MAX_SPEND_PER_JOB_CENTS") {
            p.max_spend_per_job_cents = v.parse().ok();
        }
        if let Ok(v) = std::env::var("UCX_MAX_TOTAL_SPEND_CENTS") {
            p.max_total_spend_cents = v.parse().ok();
        }
        if let Ok(v) = std::env::var("UCX_BUDGET_ALERT_THRESHOLD") {
            p.budget_alert_threshold = v.parse().unwrap_or(0.80);
        }
        if let Ok(v) = std::env::var("UCX_ALLOWED_WORKLOADS") {
            p.allowed_workloads = v.split(',').map(str::trim).map(str::to_string).collect();
        }
        if let Ok(v) = std::env::var("UCX_PREFER_NATIVE") {
            p.prefer_native = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("UCX_ALLOW_EXTERNAL") {
            p.allow_external = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("UCX_MAX_GPU_PRICE_CENTS_PER_HR") {
            p.max_gpu_price_cents_per_hr = v.parse().ok();
        }
        if let Ok(v) = std::env::var("UCX_MIN_VRAM_GB") {
            p.min_vram_gb = v.parse().ok();
        }
        if let Ok(v) = std::env::var("UCX_DENY_PROVIDERS") {
            p.deny_providers = v.split(',').map(str::trim).map(str::to_string).collect();
        }
        if let Ok(v) = std::env::var("UCX_PREFERRED_PROVIDERS") {
            p.preferred_providers = v.split(',').map(str::trim).map(str::to_string).collect();
        }

        p
    }

    /// Check whether a job spec passes this policy.
    ///
    /// Returns `Ok(())` if allowed, `Err(reason)` if rejected.
    pub fn check_job(&self, workload: &str, estimated_cost_cents: Option<u64>) -> Result<(), String> {
        // Workload allow-list check.
        if !self.allowed_workloads.is_empty()
            && !self.allowed_workloads.iter().any(|w| w.eq_ignore_ascii_case(workload))
        {
            return Err(format!("workload '{workload}' not in allowed_workloads"));
        }

        // Per-job spend limit.
        if let (Some(max), Some(cost)) = (self.max_spend_per_job_cents, estimated_cost_cents) {
            if cost > max {
                return Err(format!("estimated cost {cost} cents exceeds per-job limit {max} cents"));
            }
        }

        // Total spend limit (remaining headroom).
        if let Some(total_max) = self.max_total_spend_cents {
            let remaining = total_max.saturating_sub(self.total_spent_cents);
            if let Some(cost) = estimated_cost_cents {
                if cost > remaining {
                    return Err(format!("estimated cost {cost} cents exceeds remaining budget {remaining} cents"));
                }
            }
        }

        Ok(())
    }

    /// Returns true if total spending has crossed the alert threshold.
    pub fn is_over_budget_alert(&self) -> bool {
        if let Some(max) = self.max_total_spend_cents {
            if max > 0 {
                return (self.total_spent_cents as f64 / max as f64) >= self.budget_alert_threshold;
            }
        }
        false
    }

    /// Record that `cents` were spent on a completed job.
    pub fn record_spend(&mut self, cents: u64) {
        self.total_spent_cents = self.total_spent_cents.saturating_add(cents);
    }

    /// Build the UCX constraints JSON for a job submission, reflecting this policy.
    pub fn to_constraints_json(&self) -> serde_json::Value {
        serde_json::json!({
            "allow_external":    self.allow_external,
            "max_price_cents":   self.max_spend_per_job_cents,
            "max_queue_secs":    null,
            "regions":           [],
        })
    }

    /// Build the UCX requirements JSON fragment incorporating policy minimums.
    pub fn enrich_requirements(&self, mut req: serde_json::Value) -> serde_json::Value {
        if let Some(min_vram) = self.min_vram_gb {
            let cur = req["vram_gb"].as_f64().unwrap_or(0.0);
            req["vram_gb"] = serde_json::json!(cur.max(min_vram));
        }
        req
    }
}
