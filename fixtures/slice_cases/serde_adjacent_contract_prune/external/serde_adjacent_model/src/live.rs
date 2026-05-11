use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum LiveEnvelope {
    Started(StartedPayload),
    Update {
        sequence: Sequence,
        message: LiveMessage,
    },
    Failed(FailurePayload),
}

impl LiveEnvelope {
    pub fn summary(&self) -> String {
        match self {
            Self::Started(payload) => payload.session_id.render(),
            Self::Update { sequence, .. } => sequence.render(),
            Self::Failed(payload) => payload.code.render(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StartedPayload {
    pub session_id: SessionId,
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sequence {
    pub value: u64,
}

impl Sequence {
    pub fn render(&self) -> String {
        format!("seq:{}", self.value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionId {
    pub value: String,
}

impl SessionId {
    pub fn render(&self) -> String {
        format!("session:{}", self.value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LiveMessage {
    pub text: String,
    pub metadata: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FailurePayload {
    pub code: ErrorCode,
    pub recoverable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorCode {
    pub value: String,
}

impl ErrorCode {
    pub fn render(&self) -> String {
        format!("error:{}", self.value)
    }
}

pub fn parse_live_command(raw: &str) -> String {
    serde_json::from_str::<LiveEnvelope>(raw)
        .map(|envelope| envelope.summary())
        .unwrap_or_else(|error| format!("invalid:{error}"))
}

pub fn content() -> &'static str {
    "dead-content-meta-word"
}

pub fn kind() -> &'static str {
    "dead-kind-meta-word"
}

pub fn payload() -> &'static str {
    "dead-payload-meta-word"
}

pub fn dead_live_contract_summary(raw: &str) -> String {
    format!("dead-live-contract:{raw}")
}
