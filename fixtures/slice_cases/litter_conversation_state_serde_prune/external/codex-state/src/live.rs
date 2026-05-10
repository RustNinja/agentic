use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("deserialize conversation state: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("conversation thread missing")]
    MissingThread,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopConversationState {
    #[serde(default)]
    thread: Option<ThreadState>,
    #[serde(default)]
    pending_approvals: Vec<PendingApproval>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadState {
    title: String,
    #[serde(default)]
    turns: Vec<TurnState>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum TurnState {
    UserMessage {
        #[serde(default)]
        text: String,
    },
    AgentMessage {
        #[serde(default)]
        text: String,
    },
    ToolResult {
        #[serde(default)]
        output: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendingApproval {
    id: String,
}

pub fn conversation_preview(raw: &str) -> Result<String, ConversationError> {
    let state: DesktopConversationState = serde_json::from_str(raw)?;
    let thread = state.thread.ok_or(ConversationError::MissingThread)?;
    let visible_turns = thread
        .turns
        .iter()
        .filter(|turn| {
            matches!(
                turn,
                TurnState::UserMessage { .. } | TurnState::AgentMessage { .. }
            )
        })
        .count();
    let approval_count = state.pending_approvals.len();
    Ok(format!(
        "{}:turns={visible_turns}:approvals={approval_count}",
        thread.title
    ))
}

pub fn dead_conversation_preview(raw: &str) -> Result<String, ConversationError> {
    let state: DesktopConversationState = serde_json::from_str(raw)?;
    Ok(format!("dead:{}", state.pending_approvals.len()))
}

pub fn dead_live_helper() -> &'static str {
    "dead-live-helper"
}
