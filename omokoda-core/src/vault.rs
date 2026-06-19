use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Data Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "now_unix")]
    pub timestamp: u64,
}

fn default_confidence() -> f64 {
    1.0
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub modified_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyStar {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyData {
    pub stars: Vec<GalaxyStar>,
    pub edges: Vec<GalaxyEdge>,
    pub nebulae: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub access_level: String,
    pub auto_export: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            access_level: "private".to_string(),
            auto_export: false,
        }
    }
}

// ─── Directory Helpers ────────────────────────────────────────────────────────

pub fn vault_root() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".omokoda")
        .join("vaults")
}

pub fn vault_dir(agent_id: &str) -> PathBuf {
    vault_root().join(agent_id)
}

pub fn init_vault(agent_id: &str) -> std::io::Result<()> {
    let base = vault_dir(agent_id);
    for sub in ["knowledge", "traces", "broadcasts", "templates"] {
        std::fs::create_dir_all(base.join(sub))?;
    }
    Ok(())
}

// ─── Broadcast Template ───────────────────────────────────────────────────────

const BROADCAST_TEMPLATE: &str = r#"# Broadcast Title

## Body

Write your broadcast here.

---

**Tags**: #technosis #omokoda

**Call to Action**: ...
"#;

pub fn write_broadcast_template(agent_id: &str) -> std::io::Result<()> {
    init_vault(agent_id)?;
    let path = vault_dir(agent_id)
        .join("templates")
        .join("broadcast-template.md");
    if !path.exists() {
        std::fs::write(path, BROADCAST_TEMPLATE)?;
    }
    Ok(())
}

// ─── File Listing ─────────────────────────────────────────────────────────────

pub fn list_files(agent_id: &str) -> Vec<VaultFileEntry> {
    let base = vault_dir(agent_id);
    let mut entries = Vec::new();
    collect_md_files(&base, &base, &mut entries);
    entries
}

fn collect_md_files(base: &PathBuf, dir: &PathBuf, out: &mut Vec<VaultFileEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(base, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let meta = entry.metadata().ok();
            let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified_secs = meta
                .as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let rel = path
                .strip_prefix(base)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push(VaultFileEntry {
                path: rel,
                size_bytes,
                modified_secs,
            });
        }
    }
}

// ─── File Reading ─────────────────────────────────────────────────────────────

pub fn read_file(agent_id: &str, rel_path: &str) -> Result<String, String> {
    let base = vault_dir(agent_id);
    let target = base.join(rel_path);
    // Prevent path traversal
    let canonical = target
        .canonicalize()
        .map_err(|e| format!("file not found: {e}"))?;
    let canonical_base = base
        .canonicalize()
        .map_err(|e| format!("vault not found: {e}"))?;
    if !canonical.starts_with(&canonical_base) {
        return Err("access denied".to_string());
    }
    std::fs::read_to_string(canonical).map_err(|e| e.to_string())
}

// ─── Knowledge Triples ────────────────────────────────────────────────────────

pub fn insert_knowledge(agent_id: &str, triple: KnowledgeTriple) -> std::io::Result<()> {
    init_vault(agent_id)?;
    let path = vault_dir(agent_id)
        .join("knowledge")
        .join("triples.jsonl");
    let line = serde_json::to_string(&triple).unwrap_or_default();
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{}", line)
}

fn load_triples(agent_id: &str) -> Vec<KnowledgeTriple> {
    let path = vault_dir(agent_id)
        .join("knowledge")
        .join("triples.jsonl");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

// ─── Vault Sync (auto-export) ─────────────────────────────────────────────────

pub fn vault_sync(agent_id: &str, content: &str) -> std::io::Result<()> {
    init_vault(agent_id)?;
    let path = vault_dir(agent_id).join("traces").join("thoughts.md");
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let entry = format!("\n## {}\n\n{}\n", ts, content);
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(entry.as_bytes())
}

// ─── Galaxy ───────────────────────────────────────────────────────────────────

pub fn galaxy_data(agent_id: &str) -> GalaxyData {
    let triples = load_triples(agent_id);
    let files = list_files(agent_id);

    let mut stars: Vec<GalaxyStar> = Vec::new();
    let mut edges: Vec<GalaxyEdge> = Vec::new();

    // Each unique node (subject or object) becomes a star
    let mut node_ids: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let add_node = |label: &str,
                    kind: &str,
                    node_ids: &mut std::collections::HashMap<String, usize>,
                    stars: &mut Vec<GalaxyStar>|
     -> String {
        let id = format!("n{}", stars.len());
        if node_ids.contains_key(label) {
            let idx = node_ids[label];
            return stars[idx].id.clone();
        }
        let hash = blake3::hash(label.as_bytes());
        let b = hash.as_bytes();
        let x = (i16::from_le_bytes([b[0], b[1]]) as f64) / 100.0;
        let y = (i16::from_le_bytes([b[2], b[3]]) as f64) / 100.0;
        let z = (i16::from_le_bytes([b[4], b[5]]) as f64) / 100.0;
        node_ids.insert(label.to_string(), stars.len());
        stars.push(GalaxyStar {
            id: id.clone(),
            label: label.to_string(),
            x,
            y,
            z,
            kind: kind.to_string(),
        });
        id
    };

    for triple in &triples {
        let s_id = add_node(&triple.subject, "subject", &mut node_ids, &mut stars);
        let o_id = add_node(&triple.object, "object", &mut node_ids, &mut stars);
        edges.push(GalaxyEdge {
            from: s_id,
            to: o_id,
            label: triple.predicate.clone(),
        });
    }

    // Files become nebulae
    let nebulae = files.iter().map(|f| f.path.clone()).collect();

    GalaxyData {
        stars,
        edges,
        nebulae,
    }
}

// ─── Vault Config ─────────────────────────────────────────────────────────────

pub fn load_vault_config(agent_id: &str) -> VaultConfig {
    let path = vault_dir(agent_id).join("vault-config.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_vault_config(agent_id: &str, cfg: &VaultConfig) -> std::io::Result<()> {
    init_vault(agent_id)?;
    let path = vault_dir(agent_id).join("vault-config.json");
    let content = serde_json::to_string_pretty(cfg).unwrap_or_default();
    std::fs::write(path, content)
}
