use codex_core::{visible_events, ConversationState, ScreenStats};
use codex_protocol::{ConversationEvent, Message, Role};
use opensourced::opensourced;

#[opensourced]
pub fn render(state: &ConversationState) -> Vec<String> {
    let mut lines = vec![render_header(state.title())];
    for event in visible_events(state) {
        lines.push(render_event(event));
    }
    lines.push(render_footer(state.stats()));
    lines
}

fn render_header(title: &str) -> String {
    format!("conversation:{title}")
}

fn render_event(event: &ConversationEvent) -> String {
    match event {
        ConversationEvent::Message(message) => render_message(message),
        ConversationEvent::Status(status) => format!("status:{status}"),
        ConversationEvent::Hidden(_) => "hidden".to_string(),
    }
}

fn render_message(message: &Message) -> String {
    format!("{}:{}", role_label(message.role()), message.text())
}

fn role_label(role: Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
        Role::Tool => "tool",
    }
}

fn render_footer(stats: ScreenStats) -> String {
    format!("visible={}:total={}", stats.visible, stats.total)
}

pub fn dead_conversation_panel(state: &ConversationState) -> String {
    format!("dead:{}", state.dead_summary())
}

