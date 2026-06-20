use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crab2d_core::ProjectIoError;
use crab2d_scene::{SceneError, TilemapError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTemplate {
    EmptyProject,
    TopDownStarter,
    TilemapStarter,
}

impl ProjectTemplate {
    pub const ALL: [Self; 3] = [
        Self::EmptyProject,
        Self::TopDownStarter,
        Self::TilemapStarter,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::EmptyProject => "Empty Project",
            Self::TopDownStarter => "Top Down Starter",
            Self::TilemapStarter => "Tilemap Starter",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorProjectSession {
    pub project_path: Option<PathBuf>,
    pub project_root: Option<PathBuf>,
    pub project_name: String,
    pub dirty: bool,
}

impl EditorProjectSession {
    pub fn untitled(project_name: impl Into<String>) -> Self {
        Self {
            project_path: None,
            project_root: None,
            project_name: project_name.into(),
            dirty: false,
        }
    }

    pub fn for_path(project_name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let project_path = path.into();
        let project_root = project_path.parent().map(Path::to_path_buf);
        Self {
            project_path: Some(project_path),
            project_root,
            project_name: project_name.into(),
            dirty: false,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn set_project_file(&mut self, path: impl Into<PathBuf>) {
        let project_path = path.into();
        self.project_root = project_path.parent().map(Path::to_path_buf);
        self.project_path = Some(project_path);
        self.dirty = false;
    }

    pub fn asset_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        if let Some(root) = &self.project_root {
            roots.push(root.clone());
            roots.push(root.join("assets"));
        }
        roots
    }

    pub fn status_label(&self) -> &'static str {
        if self.dirty {
            "Unsaved changes"
        } else {
            "Saved"
        }
    }

    pub fn display_title(&self) -> String {
        if self.dirty {
            format!("Crab2D - {} *", self.project_name)
        } else {
            format!("Crab2D - {}", self.project_name)
        }
    }
}

pub fn ensure_project_structure(project_root: &Path) -> Result<(), ProjectSessionError> {
    fs::create_dir_all(project_root.join("assets/sprites"))?;
    fs::create_dir_all(project_root.join("assets/tilesets"))?;
    fs::create_dir_all(project_root.join("assets/audio"))?;
    fs::create_dir_all(project_root.join("scenes"))?;
    copy_default_sprite(project_root)?;
    Ok(())
}

fn copy_default_sprite(project_root: &Path) -> Result<(), ProjectSessionError> {
    let target = project_root.join("assets/sprites/player.png");
    if target.exists() {
        return Ok(());
    }

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/crab2d-editor/assets/sprites/player.png");
    if source.exists() {
        fs::copy(source, target)?;
    }
    Ok(())
}

#[derive(Debug)]
pub enum ProjectSessionError {
    MissingProjectPath,
    Io(std::io::Error),
    Project(ProjectIoError),
    Scene(SceneError),
    Tilemap(TilemapError),
}

impl fmt::Display for ProjectSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingProjectPath => {
                formatter.write_str("project has no save path; use Save As first")
            }
            Self::Io(error) => write!(formatter, "project session I/O failed: {error}"),
            Self::Project(error) => write!(formatter, "{error}"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::Tilemap(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ProjectSessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Project(error) => Some(error),
            Self::Scene(error) => Some(error),
            Self::Tilemap(error) => Some(error),
            Self::MissingProjectPath => None,
        }
    }
}

impl From<std::io::Error> for ProjectSessionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ProjectIoError> for ProjectSessionError {
    fn from(error: ProjectIoError) -> Self {
        Self::Project(error)
    }
}

impl From<SceneError> for ProjectSessionError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

impl From<TilemapError> for ProjectSessionError {
    fn from(error: TilemapError) -> Self {
        Self::Tilemap(error)
    }
}
