use std::collections::BTreeSet;

use super::*;

pub(super) fn asset_roots(project_roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut roots = project_roots;
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    if let Ok(current_dir) = std::env::current_dir() {
        roots.push(current_dir.join("assets"));
        roots.push(current_dir.join("apps/crab2d-editor/assets"));
    }
    roots.dedup();
    roots
}

pub(super) fn default_project_path(project_name: &str) -> String {
    let folder = sanitize_project_folder(project_name);
    PathBuf::from("projects").join(folder).display().to_string()
}

pub(super) fn default_project_file_path(project_name: &str) -> String {
    PathBuf::from(default_project_path(project_name))
        .join("project.crab2d.json")
        .display()
        .to_string()
}

pub(super) fn sanitize_project_folder(project_name: &str) -> String {
    let sanitized = project_name
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '-'
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "UntitledProject".to_owned()
    } else {
        sanitized
    }
}

pub(super) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(super) fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }

    let take = max_chars.saturating_sub(3);
    let mut compact = text.chars().take(take).collect::<String>();
    compact.push_str("...");
    compact
}

pub(super) fn format_solid_tiles(solid_tiles: &BTreeSet<u32>) -> String {
    solid_tiles
        .iter()
        .map(|tile| tile.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn parse_solid_tiles(input: &str) -> Result<BTreeSet<u32>, String> {
    let mut tiles = BTreeSet::new();
    for part in input
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let tile = part
            .parse::<u32>()
            .map_err(|_| format!("Invalid tile index: {part}"))?;
        tiles.insert(tile);
    }
    Ok(tiles)
}

pub(super) fn output_level(tone: StatusTone) -> &'static str {
    match tone {
        StatusTone::Info => "INFO",
        StatusTone::Success => "OK",
        StatusTone::Warning => "WARN",
        StatusTone::Error => "ERROR",
    }
}

pub(super) fn output_color(line: &str) -> egui::Color32 {
    let theme = theme();
    if line.contains("[ERROR]") {
        theme.colors.error
    } else if line.contains("[WARN]") {
        theme.colors.warning
    } else if line.contains("[OK]") {
        theme.colors.success
    } else {
        theme.colors.text_secondary
    }
}

pub(super) fn trim_output(output: &mut Vec<String>) {
    const MAX_LINES: usize = 200;
    if output.len() > MAX_LINES {
        let drain_count = output.len() - MAX_LINES;
        output.drain(0..drain_count);
    }
}
