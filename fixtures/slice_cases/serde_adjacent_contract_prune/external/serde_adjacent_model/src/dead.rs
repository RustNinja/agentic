use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum DeadEnvelope {
    Dead(DeadPayload),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeadPayload {
    pub value: String,
}

impl DeadPayload {
    pub fn dead_payload_debug(&self) -> String {
        format!("dead-payload:{}", self.value)
    }
}

pub fn dead_contract_summary(raw: &str) -> String {
    raw.to_string()
}
