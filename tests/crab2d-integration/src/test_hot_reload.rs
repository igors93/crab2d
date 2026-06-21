//! Integration tests for hot reload: reloading a scene from a saved project file.

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crab2d_editor::{EditorApp, EditorCommand};

    fn temp_project_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "crab2d-reload-{label}-{}-{nanos}.crab2d.json",
            std::process::id()
        ))
    }

    #[test]
    fn reload_restores_scene_to_saved_state() {
        let path = temp_project_path("restore");
        let mut app = EditorApp::new("Reload Test");
        app.open_empty_project("ReloadProject");
        app.save_current_project(&path)
            .expect("project should save");

        // Spawn a new node after saving — this should disappear on reload.
        app.execute_command_with_history(EditorCommand::create_node("Temporary"))
            .expect("node created");
        let node_count_before = app.scene_nodes().len();

        // Reload from disk — should discard the unsaved node.
        app.reload_active_scene().expect("reload should succeed");

        let node_count_after = app.scene_nodes().len();
        assert!(
            node_count_after < node_count_before,
            "reload should have removed the unsaved node ({node_count_before} → {node_count_after})"
        );
        assert!(
            !app.scene_nodes().iter().any(|n| n.name == "Temporary"),
            "the Temporary node should not exist after reload"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reload_without_saved_file_returns_error() {
        let mut app = EditorApp::new("No File Test");
        app.open_empty_project("Unsaved");

        let result = app.reload_active_scene();
        assert!(
            result.is_err(),
            "reload without a saved project path should return Err"
        );
    }

    #[test]
    fn reload_clears_undo_history() {
        let path = temp_project_path("history");
        let mut app = EditorApp::new("History Test");
        app.open_empty_project("HistoryProject");
        app.save_current_project(&path)
            .expect("project should save");

        app.execute_command_with_history(EditorCommand::create_node("Ghost"))
            .expect("command executed");
        assert!(app.can_undo(), "undo should be available before reload");

        app.reload_active_scene().expect("reload should succeed");
        assert!(
            !app.can_undo(),
            "undo history should be cleared after reload"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn reload_preserves_project_name_and_path() {
        let path = temp_project_path("metadata");
        let mut app = EditorApp::new("Meta Test");
        app.open_empty_project("MetaProject");
        app.save_current_project(&path)
            .expect("project should save");

        app.reload_active_scene().expect("reload should succeed");

        assert_eq!(
            app.project_name(),
            "MetaProject",
            "project name should survive reload"
        );
        assert_eq!(
            app.project_path(),
            Some(path.as_path()),
            "project path should survive reload"
        );

        let _ = std::fs::remove_file(&path);
    }
}
