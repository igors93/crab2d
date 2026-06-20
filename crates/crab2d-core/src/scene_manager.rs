use crab2d_scene::Scene;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SceneManager {
    pub current_path: Option<PathBuf>,
    pub stack: VecDeque<PathBuf>,
    pub pending_transition: Option<SceneTransition>,
    pub asset_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum SceneTransition {
    Replace(String),
    Push(String),
    Pop,
    Restart,
}

impl SceneManager {
    pub fn new(asset_roots: Vec<PathBuf>) -> Self {
        Self {
            current_path: None,
            stack: VecDeque::new(),
            pending_transition: None,
            asset_roots,
        }
    }

    pub fn load_scene(&mut self, path: impl Into<String>) {
        self.pending_transition = Some(SceneTransition::Replace(path.into()));
    }

    pub fn push_scene(&mut self, path: impl Into<String>) {
        self.pending_transition = Some(SceneTransition::Push(path.into()));
    }

    pub fn pop_scene(&mut self) {
        self.pending_transition = Some(SceneTransition::Pop);
    }

    pub fn restart(&mut self) {
        self.pending_transition = Some(SceneTransition::Restart);
    }

    /// Call each frame; returns the new scene if a transition happened.
    pub fn apply_transition(&mut self) -> Option<(Scene, PathBuf)> {
        let transition = self.pending_transition.take()?;
        match transition {
            SceneTransition::Replace(path) => {
                self.stack.clear();
                self.load_from_path(&path)
            }
            SceneTransition::Push(path) => {
                if let Some(current) = self.current_path.clone() {
                    self.stack.push_back(current);
                }
                self.load_from_path(&path)
            }
            SceneTransition::Pop => {
                let prev = self.stack.pop_back()?;
                self.load_from_path(&prev.display().to_string())
            }
            SceneTransition::Restart => {
                let current = self.current_path.clone()?;
                self.load_from_path(&current.display().to_string())
            }
        }
    }

    fn load_from_path(&mut self, path: &str) -> Option<(Scene, PathBuf)> {
        for root in &self.asset_roots {
            let full = root.join(path);
            if full.exists() {
                if let Ok(json) = std::fs::read_to_string(&full) {
                    if let Ok(scene) = serde_json::from_str::<Scene>(&json) {
                        self.current_path = Some(full.clone());
                        return Some((scene, full));
                    }
                }
            }
        }
        // Also try as absolute path
        let p = Path::new(path);
        if p.exists() {
            if let Ok(json) = std::fs::read_to_string(p) {
                if let Ok(scene) = serde_json::from_str::<Scene>(&json) {
                    self.current_path = Some(p.to_path_buf());
                    return Some((scene, p.to_path_buf()));
                }
            }
        }
        None
    }

    pub fn set_asset_roots(&mut self, roots: Vec<PathBuf>) {
        self.asset_roots = roots;
    }
}
