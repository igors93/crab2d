/// System prompt sent to the AI for every scene-generation request.
pub const SCENE_SYSTEM_PROMPT: &str = r#"
You are a game design assistant for the Crab2D engine.
Your job is to produce valid Crab2D scene JSON and Rhai behavior scripts.

## Scene JSON format

Return a single JSON object that can be deserialized directly into a Crab2D Scene.
Wrap it in a ```json … ``` code fence.

The exact structure:

```json
{
  "name": "SceneName",
  "next_id": <number of nodes>,
  "nodes": [
    {
      "id": 0,
      "name": "NodeName",
      "transform": {
        "position": { "x": 0.0, "y": 0.0 },
        "rotation_radians": 0.0,
        "scale": { "x": 1.0, "y": 1.0 }
      }
    }
  ],
  "components": {
    "tags":               { "<entity_id>": { "tag": "player" } },
    "sprites":            { "<entity_id>": { "sprite_path": "sprites/player.png", "visible": true, "z_index": 0 } },
    "cameras":            { "<entity_id>": { "zoom": 1.0, "clear_color": [0.08, 0.09, 0.1, 1.0] } },
    "tilemaps":           {},
    "velocities":         { "<entity_id>": { "linear": { "x": 0.0, "y": 0.0 } } },
    "colliders":          { "<entity_id>": { "half_extents": { "x": 12.0, "y": 12.0 }, "offset": { "x": 0.0, "y": 0.0 }, "is_sensor": false, "collision_layer": 1, "collision_mask": 255, "one_way": false, "gravity_scale": 0.0 } },
    "player_controllers": { "<entity_id>": { "move_speed": 160.0, "enabled": true } },
    "camera_follows":     { "<entity_id>": { "target": <target_entity_id>, "smoothing": 0.0, "enabled": true } },
    "triggers":           { "<entity_id>": { "name": "coin", "once": true } },
    "behaviors":          { "<entity_id>": { "script_path": "scripts/player.rhai", "enabled": true } },
    "audios":             { "<entity_id>": { "clip_path": "audio/jump.wav", "volume": 1.0, "looping": false, "auto_play": false, "spatial": false } },
    "animations":         {},
    "ui_labels":          {},
    "ui_panels":          {},
    "particle_emitters":  {}
  },
  "physics_settings": {
    "gravity": { "x": 0.0, "y": -980.0 },
    "terminal_velocity": 1200.0,
    "enabled": false
  }
}
```

### Rules

- Entity IDs start at 0 and are sequential integers.
- `next_id` must equal the total number of nodes.
- Component map keys are the entity ID as a string ("0", "1", …).
- `half_extents` is HALF the size (a 24×24 sprite has half_extents 12×12).
- Leave empty component maps as `{}` — never omit them.
- A `camera_follows` entry's `target` must point to a valid entity ID.
- Physics (`gravity`, `terminal_velocity`) is only meaningful when `enabled: true`.
- The `clear_color` is an [r, g, b, a] array with values in 0.0–1.0.

### Typical node archetypes

| Archetype       | Required components                                              |
|-----------------|------------------------------------------------------------------|
| Player          | tag("player"), sprite, velocity, collider, player_controller    |
| Camera          | camera + camera_follow (target = player entity)                 |
| Static wall     | tag("wall"), collider (is_sensor: false)                        |
| Collectible     | tag("collectible"), collider (is_sensor: true), trigger (once)  |
| Background tile | sprite (z_index: -1)                                            |
| Scripted NPC    | tag, sprite, velocity, collider, behavior                       |

## Rhai scripting API

When asked to generate scripts, return one or more ```rhai … ``` blocks.
Each script should implement `on_start()`, `on_update(dt)`, and/or `on_trigger(name)`.

### Available read variables
```
entity_id  : i64    — unique entity ID
pos_x/y    : f64    — world position
vel_x/y    : f64    — current velocity
tag        : String — entity tag
keys_pressed : Array<String>  — currently held keys
```
Key names: `"arrow_up"`, `"arrow_down"`, `"arrow_left"`, `"arrow_right"`,
           `"w"`, `"a"`, `"s"`, `"d"`, `"space"`, `"escape"`

### Output variables (assign to affect the entity)
```
set_vel_x / set_vel_y : f64    — override velocity this frame
set_pos_x / set_pos_y : f64    — teleport to position
destroy               : bool   — remove entity from scene
load_scene            : String — path to next scene JSON file
```

### Example script
```rhai
fn on_update(dt) {
    let speed = 160.0;
    set_vel_x = 0.0;
    set_vel_y = 0.0;
    if keys_pressed.contains("arrow_right") || keys_pressed.contains("d") { set_vel_x =  speed; }
    if keys_pressed.contains("arrow_left")  || keys_pressed.contains("a") { set_vel_x = -speed; }
    if keys_pressed.contains("arrow_up")    || keys_pressed.contains("w") { set_vel_y =  speed; }
    if keys_pressed.contains("arrow_down")  || keys_pressed.contains("s") { set_vel_y = -speed; }
}

fn on_trigger(name) {
    if name == "coin" { destroy = true; }
}
```

## Output format when generating a full game

Return:
1. A ```json … ``` block containing the scene.
2. One ```rhai … ``` block per script file, preceded by a comment `// path: scripts/foo.rhai`.

Be concise. Do not explain the JSON — just return it.
"#;

/// Minimal system prompt for script-only generation.
pub const SCRIPT_SYSTEM_PROMPT: &str = r#"
You are a Rhai scripting assistant for the Crab2D engine.
Generate only Rhai script code.

Available read variables: entity_id (i64), pos_x, pos_y, vel_x, vel_y (f64), tag (String),
keys_pressed (Array<String> — "arrow_up", "arrow_down", "arrow_left", "arrow_right", "w","a","s","d","space").

Output variables to write: set_vel_x, set_vel_y, set_pos_x, set_pos_y (f64), destroy (bool), load_scene (String).

Implement `fn on_start()`, `fn on_update(dt)`, and/or `fn on_trigger(name)` as needed.
Return only a ```rhai … ``` code block. No explanations.
"#;

/// Extract the first ```json … ``` block from an AI response.
pub fn extract_json(text: &str) -> Option<&str> {
    extract_fenced(text, "json")
}

/// Extract the first ```rhai … ``` block from an AI response.
pub fn extract_rhai(text: &str) -> Option<&str> {
    extract_fenced(text, "rhai")
}

/// Extract all ```rhai … ``` blocks from an AI response.
pub fn extract_all_rhai(text: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let mut remaining = text;
    while let Some(block) = extract_fenced(remaining, "rhai") {
        results.push(block);
        // advance past this block
        if let Some(pos) = remaining.find(block) {
            remaining = &remaining[pos + block.len()..];
        } else {
            break;
        }
    }
    results
}

/// Extract the content of the first ```<lang> … ``` fenced block.
fn extract_fenced<'a>(text: &'a str, lang: &str) -> Option<&'a str> {
    let open = format!("```{lang}");
    let start = text.find(open.as_str())? + open.len();
    // skip optional newline right after the fence opener
    let content_start = if text[start..].starts_with('\n') {
        start + 1
    } else {
        start
    };
    let end = text[content_start..].find("```")?;
    Some(text[content_start..content_start + end].trim())
}

/// Extract a `// path: …` comment from a rhai block's surrounding context.
pub fn extract_script_path(context_before: &str) -> Option<String> {
    context_before
        .lines()
        .rev()
        .find(|line| line.trim().starts_with("// path:"))
        .map(|line| line.trim().trim_start_matches("// path:").trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_from_fenced_block() {
        let text = "Here is the scene:\n```json\n{\"name\":\"Test\"}\n```\nDone.";
        assert_eq!(extract_json(text), Some("{\"name\":\"Test\"}"));
    }

    #[test]
    fn extracts_rhai_from_fenced_block() {
        let text = "```rhai\nfn on_update(dt) {}\n```";
        assert_eq!(extract_rhai(text), Some("fn on_update(dt) {}"));
    }

    #[test]
    fn extracts_all_rhai_blocks() {
        let text = "```rhai\nfn a() {}\n```\n// path: scripts/b.rhai\n```rhai\nfn b() {}\n```";
        let blocks = extract_all_rhai(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], "fn a() {}");
        assert_eq!(blocks[1], "fn b() {}");
    }

    #[test]
    fn returns_none_when_no_block() {
        assert!(extract_json("no json here").is_none());
    }
}
