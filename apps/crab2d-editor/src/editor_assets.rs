use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use eframe::egui;

pub struct EditorTextureCache {
    asset_root: PathBuf,
    textures: BTreeMap<String, TextureLoadState>,
}

impl EditorTextureCache {
    pub fn new(asset_root: impl Into<PathBuf>) -> Self {
        Self {
            asset_root: asset_root.into(),
            textures: BTreeMap::new(),
        }
    }

    pub fn load(&mut self, ctx: &egui::Context, asset_path: &str) -> TextureLookup<'_> {
        if asset_path.trim().is_empty() {
            return TextureLookup::Missing;
        }

        let normalized = normalize_asset_path(asset_path);
        let asset_root = self.asset_root.clone();

        let state = match self.textures.entry(normalized.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let state = match load_texture(ctx, &asset_root, &normalized) {
                    Ok(texture) => TextureLoadState::Loaded(texture),
                    Err(error) => TextureLoadState::Failed(error.to_string()),
                };
                entry.insert(state)
            }
        };

        TextureLookup::from_state(state)
    }
}

fn load_texture(
    ctx: &egui::Context,
    asset_root: &Path,
    normalized_path: &str,
) -> Result<egui::TextureHandle, TextureLoadError> {
    let path = resolve_path(asset_root, normalized_path);
    let image = image::open(&path).map_err(|source| TextureLoadError {
        path: path.clone(),
        message: source.to_string(),
    })?;

    let image = image.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

    Ok(ctx.load_texture(
        format!("asset:{normalized_path}"),
        color_image,
        egui::TextureOptions::NEAREST,
    ))
}

fn resolve_path(asset_root: &Path, normalized_path: &str) -> PathBuf {
    let path = Path::new(normalized_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        let editor_asset = asset_root.join(path);
        if editor_asset.exists() {
            editor_asset
        } else {
            path.to_path_buf()
        }
    }
}

enum TextureLoadState {
    Loaded(egui::TextureHandle),
    Failed(String),
}

pub enum TextureLookup<'a> {
    Loaded(&'a egui::TextureHandle),
    Failed(&'a str),
    Missing,
}

impl<'a> TextureLookup<'a> {
    fn from_state(state: &'a TextureLoadState) -> Self {
        match state {
            TextureLoadState::Loaded(texture) => Self::Loaded(texture),
            TextureLoadState::Failed(error) => Self::Failed(error),
        }
    }
}

#[derive(Debug)]
struct TextureLoadError {
    path: PathBuf,
    message: String,
}

impl std::fmt::Display for TextureLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to load texture '{}': {}",
            self.path.display(),
            self.message
        )
    }
}

fn normalize_asset_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_owned()
}
