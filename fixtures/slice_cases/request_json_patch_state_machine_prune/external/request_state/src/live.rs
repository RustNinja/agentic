use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug)]
pub enum PatchError {
    Json(String),
    MissingPath,
}

impl From<serde_json::Error> for PatchError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchRequest {
    pub request_id: String,
    pub command: PatchCommand,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op", content = "value", rename_all = "camelCase")]
pub enum PatchCommand {
    Add(PatchPayload),
    Replace(PatchPayload),
    Remove(RemovePayload),
    Test(TestPayload),
}

impl PatchCommand {
    fn path_label(&self) -> Option<String> {
        match self {
            Self::Add(payload) | Self::Replace(payload) => payload.path_label(),
            Self::Remove(payload) => payload.path.first().map(PatchSegment::render),
            Self::Test(payload) => payload.path.first().map(PatchSegment::render),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPayload {
    pub path: Vec<PatchSegment>,
    pub value: Value,
}

impl PatchPayload {
    fn path_label(&self) -> Option<String> {
        self.path.first().map(PatchSegment::render)
    }

    fn dead_payload_debug(&self) -> String {
        format!("dead-payload:{}", self.path.len())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePayload {
    pub path: Vec<PatchSegment>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPayload {
    pub path: Vec<PatchSegment>,
    pub expected: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSegment {
    pub key: String,
}

impl PatchSegment {
    fn render(&self) -> String {
        self.key.clone()
    }

    fn dead_segment_debug(&self) -> String {
        format!("dead-segment:{}", self.key)
    }
}

pub fn apply_patch_request(raw: &str) -> Result<String, PatchError> {
    let request: PatchRequest = serde_json::from_str(raw)?;
    let path = request.command.path_label().ok_or(PatchError::MissingPath)?;
    let mut document = Map::new();
    match request.command {
        PatchCommand::Add(payload) | PatchCommand::Replace(payload) => {
            document.insert(path.clone(), payload.value);
        }
        PatchCommand::Remove(_) => {
            document.remove(&path);
        }
        PatchCommand::Test(payload) => {
            return Ok(format!(
                "{}:test:{path}:{}",
                request.request_id,
                payload.expected.is_null()
            ));
        }
    }
    Ok(format!("{}:{path}:{}", request.request_id, document.len()))
}

pub fn dead_live_state_summary(raw: &str) -> String {
    let payload = PatchPayload {
        path: vec![PatchSegment {
            key: raw.to_string(),
        }],
        value: Value::Null,
    };
    format!("{}:{}", payload.dead_payload_debug(), payload.path[0].dead_segment_debug())
}
