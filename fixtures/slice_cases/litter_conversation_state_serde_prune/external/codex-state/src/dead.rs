use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeadConversationError {
    #[error("dead deserialize: {0}")]
    Deserialize(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeadConversationState {
    label: String,
}

pub fn dead_conversation_preview(raw: &str) -> Result<String, DeadConversationError> {
    let state: DeadConversationState = serde_json::from_str(raw)?;
    Ok(format!("dead:{}", state.label))
}

