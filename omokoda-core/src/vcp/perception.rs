//! Active Perception Loop — continuous VCP device health monitoring.
//! Migrated from sovereign-node. Belongs in Omo-Koda2 (agent OS layer).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, info};

use vcp::DeviceRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceHealth {
    Online,
    Stale,
    Offline,
}

impl DeviceHealth {
    pub fn label(&self) -> &'static str {
        match self {
            DeviceHealth::Online  => "online",
            DeviceHealth::Stale   => "stale",
            DeviceHealth::Offline => "offline",
        }
    }
}

/// A thin status event emitted by the perception loop.
#[derive(Debug, Clone)]
pub struct StatusUpdate {
    pub device_id: String,
    pub health:    DeviceHealth,
    pub age_secs:  u64,
    pub message:   String,
}

pub fn spawn_perception_loop(
    registry:             DeviceRegistry,
    events_tx:            broadcast::Sender<StatusUpdate>,
    interval_secs:        u64,
    stale_threshold_secs: u64,
    http_client:          Arc<reqwest::Client>,
) {
    tokio::spawn(async move {
        let mut health_map: HashMap<String, DeviceHealth> = HashMap::new();
        let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        info!(interval_secs, stale_threshold_secs, "perception loop started");

        loop {
            tick.tick().await;
            let devices = registry.all().await;
            debug!(count = devices.len(), "perception tick");

            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            for device in &devices {
                let age_secs = now_ms.saturating_sub(device.last_seen_ms) / 1000;

                let new_health = if age_secs > stale_threshold_secs * 3 {
                    if device.device_id.starts_with("unitree:go2:") {
                        let ip = device.device_id.strip_prefix("unitree:go2:").unwrap_or("unknown");
                        let url = format!("http://{ip}:8080/api/v1/health");
                        match http_client.get(&url).timeout(Duration::from_secs(2)).send().await {
                            Ok(r) if r.status().is_success() => DeviceHealth::Online,
                            _ => DeviceHealth::Offline,
                        }
                    } else {
                        DeviceHealth::Offline
                    }
                } else if age_secs > stale_threshold_secs {
                    DeviceHealth::Stale
                } else {
                    DeviceHealth::Online
                };

                if health_map.get(&device.device_id) != Some(&new_health) {
                    let msg = format!(
                        "Device {} → {} (last seen {}s ago)",
                        device.device_id,
                        new_health.label(),
                        age_secs,
                    );
                    let _ = events_tx.send(StatusUpdate {
                        device_id: device.device_id.clone(),
                        health:    new_health.clone(),
                        age_secs,
                        message:   msg,
                    });
                    health_map.insert(device.device_id.clone(), new_health);
                }
            }

            health_map.retain(|id, _| devices.iter().any(|d| &d.device_id == id));
        }
    });
}
