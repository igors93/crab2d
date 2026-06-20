#[derive(Debug)]
pub struct EngineContext {
    pub app_name: String,
    registered_tools: Vec<String>,
}

impl EngineContext {
    pub fn new(app_name: String) -> Self {
        Self {
            app_name,
            registered_tools: Vec::new(),
        }
    }

    pub fn register_tool(&mut self, name: impl Into<String>) {
        self.registered_tools.push(name.into());
    }

    pub fn registered_tools(&self) -> &[String] {
        &self.registered_tools
    }
}

pub trait Plugin {
    fn name(&self) -> &str;
    fn register(&mut self, context: &mut EngineContext);
}
