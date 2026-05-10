use opensourced::opensourced;

#[opensourced]
pub fn conversation_preview(raw: &str) -> Result<String, codex_state::ConversationError> {
    codex_state::conversation_preview(raw)
}

pub fn dead_conversation_preview(raw: &str) -> Result<String, codex_state::DeadConversationError> {
    codex_state::dead_conversation_preview(raw)
}
