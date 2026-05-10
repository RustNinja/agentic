#[derive(Clone, Copy)]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Clone)]
pub struct Message {
    role: Role,
    text: String,
}

impl Message {
    pub fn new(role: Role, text: &str) -> Self {
        Self {
            role,
            text: text.to_string(),
        }
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn dead_render(&self) -> String {
        format!("dead:{}", self.text)
    }
}

#[derive(Clone)]
pub enum ConversationEvent {
    Message(Message),
    Status(String),
    Hidden(String),
}

impl ConversationEvent {
    pub fn message(role: Role, text: &str) -> Self {
        Self::Message(Message::new(role, text))
    }

    pub fn status(value: &str) -> Self {
        Self::Status(value.to_string())
    }

    pub fn hidden(value: &str) -> Self {
        Self::Hidden(value.to_string())
    }

    pub fn is_visible(&self) -> bool {
        !matches!(self, Self::Hidden(_))
    }

    pub fn dead_kind(&self) -> &'static str {
        "dead-kind"
    }
}

pub fn dead_live_event() -> ConversationEvent {
    ConversationEvent::hidden("dead")
}

