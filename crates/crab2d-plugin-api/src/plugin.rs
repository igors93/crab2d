use crate::EngineContext;

pub trait Plugin {
    fn name(&self) -> &str;
    fn register(&mut self, context: &mut EngineContext);
}
