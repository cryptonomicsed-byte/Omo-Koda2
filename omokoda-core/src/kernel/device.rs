use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeviceKind {
    Gpu,
    Npu,
    Cpu,
    Camera,
    Microphone,
    Speaker,
    Screen,
    Network,
    Drone,
    Robot,
    Sensor,
    LoRa,
    Nfc,
    Ble,
    Custom(String),
}

impl std::fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gpu     => write!(f, "gpu"),
            Self::Npu     => write!(f, "npu"),
            Self::Cpu     => write!(f, "cpu"),
            Self::Camera  => write!(f, "camera"),
            Self::Microphone => write!(f, "microphone"),
            Self::Speaker => write!(f, "speaker"),
            Self::Screen  => write!(f, "screen"),
            Self::Network => write!(f, "network"),
            Self::Drone   => write!(f, "drone"),
            Self::Robot   => write!(f, "robot"),
            Self::Sensor  => write!(f, "sensor"),
            Self::LoRa    => write!(f, "lora"),
            Self::Nfc     => write!(f, "nfc"),
            Self::Ble     => write!(f, "ble"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DeviceStatus {
    Available,
    InUse { by_agent: String },
    Leased { lease_id: String, to_agent: String },
    Offline,
    Faulted { reason: String },
}

#[derive(Debug, Clone)]
pub struct GpuProfile {
    pub vram_mb:       u64,
    pub compute_units: u32,
    pub current_load:  f32,   // 0.0 – 1.0
    pub energy_w:      f32,
    pub temp_c:        f32,
    pub hardware_id:   String,
    pub attestation:   Option<String>,
    pub reputation:    f64,
}

#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    pub device_id:   String,   // canonical path: /devices/gpu/0001
    pub kind:        DeviceKind,
    pub owner:       String,   // agent_id that registered this device
    pub model:       String,
    pub status:      DeviceStatus,
    pub gpu_profile: Option<GpuProfile>,
    pub registered_at: u64,
}

impl DeviceDescriptor {
    pub fn new(device_id: impl Into<String>, kind: DeviceKind, owner: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            kind,
            owner: owner.into(),
            model: model.into(),
            status: DeviceStatus::Available,
            gpu_profile: None,
            registered_at: now_secs(),
        }
    }

    pub fn path(&self) -> &str {
        &self.device_id
    }
}

pub struct DeviceTree {
    devices: RwLock<HashMap<String, DeviceDescriptor>>,
    counters: RwLock<HashMap<String, u32>>,  // kind → next id
}

impl DeviceTree {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            devices: RwLock::new(HashMap::new()),
            counters: RwLock::new(HashMap::new()),
        })
    }

    fn next_id(&self, kind: &DeviceKind) -> String {
        let key = kind.to_string();
        let mut counters = self.counters.write().unwrap();
        let n = counters.entry(key.clone()).or_insert(0);
        *n += 1;
        format!("/devices/{key}/{:04}", n)
    }

    pub fn register(&self, kind: DeviceKind, owner: impl Into<String>, model: impl Into<String>, gpu_profile: Option<GpuProfile>) -> String {
        let path = self.next_id(&kind);
        let mut dev = DeviceDescriptor::new(path.clone(), kind, owner, model);
        dev.gpu_profile = gpu_profile;
        self.devices.write().unwrap().insert(path.clone(), dev);
        path
    }

    pub fn get(&self, device_id: &str) -> Option<DeviceDescriptor> {
        self.devices.read().unwrap().get(device_id).cloned()
    }

    pub fn set_status(&self, device_id: &str, status: DeviceStatus) -> bool {
        let mut map = self.devices.write().unwrap();
        if let Some(d) = map.get_mut(device_id) {
            d.status = status;
            true
        } else {
            false
        }
    }

    pub fn list_by_kind(&self, kind: &DeviceKind) -> Vec<DeviceDescriptor> {
        self.devices.read().unwrap()
            .values()
            .filter(|d| &d.kind == kind)
            .cloned()
            .collect()
    }

    pub fn list_all(&self) -> Vec<DeviceDescriptor> {
        self.devices.read().unwrap().values().cloned().collect()
    }

    pub fn deregister(&self, device_id: &str) {
        self.devices.write().unwrap().remove(device_id);
    }
}

/// Facade that owns the DeviceTree and adds higher-level operations
pub struct DeviceManager {
    pub tree: Arc<DeviceTree>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self { tree: DeviceTree::new() }
    }

    pub fn register_gpu(&self, owner: impl Into<String>, model: impl Into<String>, profile: GpuProfile) -> String {
        self.tree.register(DeviceKind::Gpu, owner, model, Some(profile))
    }

    pub fn register_drone(&self, owner: impl Into<String>, model: impl Into<String>) -> String {
        self.tree.register(DeviceKind::Drone, owner, model, None)
    }

    pub fn register_robot(&self, owner: impl Into<String>, model: impl Into<String>) -> String {
        self.tree.register(DeviceKind::Robot, owner, model, None)
    }

    pub fn available_gpus(&self) -> Vec<DeviceDescriptor> {
        self.tree.list_by_kind(&DeviceKind::Gpu)
            .into_iter()
            .filter(|d| matches!(d.status, DeviceStatus::Available))
            .collect()
    }
}

impl Default for DeviceManager {
    fn default() -> Self { Self::new() }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
