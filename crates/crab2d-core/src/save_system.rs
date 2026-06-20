use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveData {
    pub version: u32,
    pub data: HashMap<String, Value>,
}

impl SaveData {
    pub fn new() -> Self {
        Self {
            version: 1,
            data: HashMap::new(),
        }
    }

    pub fn set_int(&mut self, key: &str, value: i64) {
        self.data.insert(key.to_string(), Value::from(value));
    }
    pub fn set_float(&mut self, key: &str, value: f64) {
        self.data.insert(key.to_string(), Value::from(value));
    }
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.data.insert(key.to_string(), Value::from(value));
    }
    pub fn set_string(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), Value::from(value));
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_i64()
    }
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.data.get(key)?.as_f64()
    }
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }
}

pub struct GameSave {
    save_dir: PathBuf,
}

impl GameSave {
    pub fn new(save_dir: impl Into<PathBuf>) -> Self {
        let dir = save_dir.into();
        let _ = std::fs::create_dir_all(&dir);
        Self { save_dir: dir }
    }

    pub fn save(&self, slot: u32, data: &SaveData) -> Result<(), String> {
        let path = self.slot_path(slot);
        let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn load(&self, slot: u32) -> Result<SaveData, String> {
        let path = self.slot_path(slot);
        if !path.exists() {
            return Ok(SaveData::new());
        }
        let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    pub fn delete(&self, slot: u32) -> Result<(), String> {
        let path = self.slot_path(slot);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn exists(&self, slot: u32) -> bool {
        self.slot_path(slot).exists()
    }

    fn slot_path(&self, slot: u32) -> PathBuf {
        self.save_dir.join(format!("save_{slot:02}.json"))
    }
}
