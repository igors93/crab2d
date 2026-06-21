// test_prefabs.rs — testes de round-trip de prefab em ProjectDocument

#[cfg(test)]
mod tests {
    use crab2d_core::{Engine, EngineConfig, ProjectDocument};
    use crab2d_editor::{EditorCommand, EditorCommandResult};
    use crab2d_scene::{SpriteComponent, TagComponent, Transform2D};

    fn test_engine() -> Engine {
        Engine::new(EngineConfig::new("Crab2D Prefab Test"))
    }

    #[test]
    fn prefab_survives_project_document_round_trip() {
        let mut engine = test_engine();

        // Create an entity with some components
        let hero = engine.active_scene.spawn_node("Hero");
        engine
            .active_scene
            .add_tag(hero, TagComponent::new("hero"))
            .expect("tag should attach");
        engine
            .active_scene
            .add_sprite(hero, SpriteComponent::new("sprites/hero.png"))
            .expect("sprite should attach");

        // Capture as prefab via editor command
        EditorCommand::create_prefab_from_entity(hero, "HeroPrefab")
            .apply(&mut engine)
            .expect("create prefab should succeed");

        assert!(engine.prefabs.get("HeroPrefab").is_some());

        // Save and reload project
        let json = engine
            .project_document()
            .to_json_string()
            .expect("should serialize");
        let loaded = ProjectDocument::from_json_str(&json).expect("should deserialize");

        // Prefab must still be in the registry
        let prefab = loaded
            .prefabs
            .get("HeroPrefab")
            .expect("prefab should survive round-trip");
        assert_eq!(prefab.name, "HeroPrefab");
        assert!(prefab.tag.is_some());
        assert!(prefab.sprite.is_some());
        assert_eq!(
            prefab.sprite.as_ref().expect("sprite").sprite_path,
            "sprites/hero.png"
        );
    }

    #[test]
    fn instantiated_prefab_entity_has_same_components_as_source() {
        let mut engine = test_engine();

        // Build source entity
        let source = engine.active_scene.spawn_node("Coin");
        engine
            .active_scene
            .add_tag(source, TagComponent::new("collectible"))
            .expect("tag should attach");
        engine
            .active_scene
            .add_sprite(source, SpriteComponent::new("sprites/coin.png"))
            .expect("sprite should attach");

        // Register prefab
        EditorCommand::create_prefab_from_entity(source, "CoinPrefab")
            .apply(&mut engine)
            .expect("create prefab should succeed");

        // Instantiate at a different position
        let result = EditorCommand::instantiate_prefab("CoinPrefab", Transform2D::default())
            .apply(&mut engine)
            .expect("instantiate should succeed");

        let EditorCommandResult::CreatedNode(new_entity) = result else {
            panic!("instantiate should return the created entity id");
        };

        // Verify components match source
        assert!(engine.active_scene.tag(new_entity).is_some());
        assert_eq!(
            engine.active_scene.tag(new_entity).expect("tag exists").tag,
            "collectible"
        );
        assert!(engine.active_scene.sprite(new_entity).is_some());
        assert_eq!(
            engine
                .active_scene
                .sprite(new_entity)
                .expect("sprite exists")
                .sprite_path,
            "sprites/coin.png"
        );
    }
}
