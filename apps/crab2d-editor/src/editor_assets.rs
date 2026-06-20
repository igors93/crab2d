use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use eframe::egui;

pub struct EditorTextureCache {
    asset_root: PathBuf,
    // Only successful loads are cached; failures are retried each frame so that
    // fixing or adding a missing file takes effect without restarting the editor.
    textures: BTreeMap<String, egui::TextureHandle>,
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

        if !self.textures.contains_key(&normalized) {
            let asset_root = self.asset_root.clone();
            match load_texture(ctx, &asset_root, &normalized) {
                Ok(texture) => {
                    self.textures.insert(normalized.clone(), texture);
                }
                // Do NOT cache failures: retry next frame so that fixing or
                // adding a missing file takes effect without restarting.
                Err(error) => return TextureLookup::Failed(error.to_string()),
            }
        }

        TextureLookup::Loaded(self.textures.get(&normalized).expect("just inserted"))
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

pub enum TextureLookup<'a> {
    Loaded(&'a egui::TextureHandle),
    /// Error message is owned because failed loads are not cached.
    Failed(String),
    Missing,
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
