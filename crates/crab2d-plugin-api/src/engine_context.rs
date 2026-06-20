#[derive(Debug)]
pub struct EngineContext {
    pub app_name: String,
    registered_tools: Vec<String>,
}

impl EngineContext {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            registered_tools: Vec::new(),
        }
    }

    pub fn register_tool(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !name.is_empty() {
            self.registered_tools.push(name);
        }
    }

    pub fn registered_tools(&self) -> &[String] {
        &self.registered_tools
    }
}

#[cfg(test)]
mod tests {
    use crate::EngineContext;

    #[test]
    fn empty_tool_names_are_ignored() {
        let mut context = EngineContext::new("Crab2D");

        context.register_tool("");
        context.register_tool("Tile Brush");

        assert_eq!(context.registered_tools(), &["Tile Brush".to_string()]);
    }
}
