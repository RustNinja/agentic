use codex_protocol::ConversationEvent;

#[derive(Clone, Copy)]
pub struct ScreenStats {
    pub visible: usize,
    pub total: usize,
}

pub struct ConversationState {
    title: String,
    events: Vec<ConversationEvent>,
}

impl ConversationState {
    pub fn new(title: &str, events: Vec<ConversationEvent>) -> Self {
        Self {
            title: title.to_string(),
            events,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn events(&self) -> &[ConversationEvent] {
        &self.events
    }

    pub fn stats(&self) -> ScreenStats {
        ScreenStats {
            visible: self.events.iter().filter(|event| event.is_visible()).count(),
            total: self.events.len(),
        }
    }

    pub fn dead_summary(&self) -> String {
        format!("dead:{}", self.title)
    }
}

pub fn visible_events(state: &ConversationState) -> impl Iterator<Item = &ConversationEvent> {
    state.events().iter().filter(|event| event.is_visible())
}

pub fn dead_live_state() -> ConversationState {
    ConversationState::new("dead", Vec::new())
}

