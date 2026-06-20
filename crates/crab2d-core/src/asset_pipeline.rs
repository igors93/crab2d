use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetKind {
    Image,
    Audio,
    Script,
    Scene,
    Font,
    Unknown,
}

impl AssetKind {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "webp" => Self::Image,
            "wav" | "ogg" | "mp3" | "flac" => Self::Audio,
            "rhai" | "lua" | "js" => Self::Script,
            "json" | "crab2d" => Self::Scene,
            "ttf" | "otf" => Self::Font,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    pub id: Uuid,
    pub path: String,
    pub kind: AssetKind,
    pub imported_at: u64,
}

impl AssetMeta {
    pub fn new(path: impl Into<String>, kind: AssetKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            path: path.into(),
            kind,
            imported_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssetHandle(pub Uuid);

impl AssetHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AssetHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct AssetRegistry {
    by_id: HashMap<Uuid, AssetMeta>,
    by_path: HashMap<String, Uuid>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Import or re-register an asset at the given path.
    pub fn import(&mut self, path: &str) -> AssetHandle {
        if let Some(&id) = self.by_path.get(path) {
            return AssetHandle(id);
        }
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let kind = AssetKind::from_extension(ext);
        let meta = AssetMeta::new(path, kind);
        let id = meta.id;
        self.by_path.insert(path.to_string(), id);
        self.by_id.insert(id, meta);
        AssetHandle(id)
    }

    /// Scan a directory tree and import all recognized assets.
    pub fn scan_directory(&mut self, root: &Path) {
        if !root.is_dir() {
            return;
        }
        let Ok(entries) = std::fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_directory(&path);
            } else if let Some(rel) = path.strip_prefix(root).ok().and_then(|p| p.to_str()) {
                self.import(rel);
            }
        }
    }

    pub fn resolve_path(&self, handle: &AssetHandle) -> Option<&str> {
        self.by_id.get(&handle.0).map(|m| m.path.as_str())
    }

    pub fn get_meta(&self, handle: &AssetHandle) -> Option<&AssetMeta> {
        self.by_id.get(&handle.0)
    }

    pub fn handle_for_path(&self, path: &str) -> Option<AssetHandle> {
        self.by_path.get(path).map(|&id| AssetHandle(id))
    }

    pub fn all_of_kind(&self, kind: AssetKind) -> Vec<&AssetMeta> {
        self.by_id.values().filter(|m| m.kind == kind).collect()
    }

    /// Save registry as a JSON file alongside assets.
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.by_id).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Load registry from a JSON file.
    pub fn load_from(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let by_id: HashMap<Uuid, AssetMeta> =
            serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let by_path = by_id
            .iter()
            .map(|(id, meta)| (meta.path.clone(), *id))
            .collect();
        Ok(Self { by_id, by_path })
    }
}
