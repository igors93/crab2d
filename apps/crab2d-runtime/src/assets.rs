use std::path::{Path, PathBuf};

pub(crate) fn asset_roots(project_path: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = project_path.parent() {
        roots.push(parent.to_path_buf());
        roots.push(parent.join("assets"));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
        roots.push(current_dir.join("apps/crab2d-editor/assets"));
        roots.push(current_dir.join("apps/crab2d-runtime/assets"));
    }
    roots
}

pub(crate) fn resolve_path(asset_roots: &[PathBuf], normalized_path: &str) -> PathBuf {
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

pub(crate) fn normalize_asset_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_owned()
}
