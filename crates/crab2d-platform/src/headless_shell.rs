use crate::{PlatformEvent, PlatformShell};

#[derive(Debug, Default)]
pub struct HeadlessShell {
    queued_events: Vec<PlatformEvent>,
}

impl HeadlessShell {
    pub fn push_event(&mut self, event: PlatformEvent) {
        self.queued_events.push(event);
    }
}

impl PlatformShell for HeadlessShell {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        std::mem::take(&mut self.queued_events)
    }
}

#[cfg(test)]
mod tests {
    use crate::{HeadlessShell, KeyCode, PlatformEvent, PlatformShell};

    #[test]
    fn headless_shell_returns_queued_events_once() {
        let mut shell = HeadlessShell::default();
        shell.push_event(PlatformEvent::KeyPressed(KeyCode::Space));

        assert_eq!(
            shell.poll_events(),
            vec![PlatformEvent::KeyPressed(KeyCode::Space)]
        );
        assert!(shell.poll_events().is_empty());
    }
}
