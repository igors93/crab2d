use crate::PlatformEvent;

pub trait PlatformShell {
    fn poll_events(&mut self) -> Vec<PlatformEvent>;
}
