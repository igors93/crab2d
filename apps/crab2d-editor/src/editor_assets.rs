use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui;

pub struct EditorTextureCache {
    asset_roots: Vec<PathBuf>,
    textures: BTreeMap<String, egui::TextureHandle>,
}

impl EditorTextureCache {
    pub fn new(asset_roots: impl Into<Vec<PathBuf>>) -> Self {
        Self {
            asset_roots: asset_roots.into(),
            textures: BTreeMap::new(),
        }
    }

    pub fn load(&mut self, ctx: &egui::Context, asset_path: &str) -> TextureLookup<'_> {
        if asset_path.trim().is_empty() {
            return TextureLookup::Missing;
        }

        let normalized = normalize_asset_path(asset_path);

        if !self.textures.contains_key(&normalized) {
            match load_texture(ctx, &self.asset_roots, &normalized) {
                Ok(texture) => {
                    self.textures.insert(normalized.clone(), texture);
                }
                // Failed loads are retried later so newly added files appear without restart.
                Err(error) => return TextureLookup::Failed(error.to_string()),
            }
        }

        TextureLookup::Loaded(self.textures.get(&normalized).expect("texture exists"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageAsset {
    pub display_name: String,
    pub asset_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageAssetCatalog {
    images: Vec<ImageAsset>,
}

impl ImageAssetCatalog {
    pub fn scan(asset_roots: &[PathBuf]) -> Self {
        let mut deduped = BTreeMap::new();

        for root in asset_roots {
            scan_root(root, root, &mut deduped);
        }

        Self {
            images: deduped.into_values().collect(),
        }
    }

    pub fn images(&self) -> &[ImageAsset] {
        &self.images
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

fn scan_root(root: &Path, current: &Path, images: &mut BTreeMap<String, ImageAsset>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_root(root, &path, images);
            continue;
        }
        if !is_supported_image(&path) {
            continue;
        }

        let asset_path = path
            .strip_prefix(root)
            .ok()
            .and_then(|path| path.to_str())
            .map(normalize_asset_path)
            .unwrap_or_else(|| normalize_asset_path(path.to_string_lossy()));
        let display_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image")
            .to_owned();

        images.entry(asset_path.clone()).or_insert(ImageAsset {
            display_name,
            asset_path,
        });
    }
}

fn load_texture(
    ctx: &egui::Context,
    asset_roots: &[PathBuf],
    normalized_path: &str,
) -> Result<egui::TextureHandle, TextureLoadError> {
    let path = resolve_path(asset_roots, normalized_path);
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

fn resolve_path(asset_roots: &[PathBuf], normalized_path: &str) -> PathBuf {
    let path = Path::new(normalized_path);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    for root in asset_roots {
        let candidate = root.join(path);
        if candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

fn is_supported_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp")
    )
}

pub enum TextureLookup<'a> {
    Loaded(&'a egui::TextureHandle),
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

pub fn normalize_asset_path(path: impl AsRef<str>) -> String {
    path.as_ref()
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_owned()
}
