#[derive(Clone, Copy)]
pub enum Method {
    Init,
    Status,
    Shutdown,
}

impl Method {
    pub fn from_wire(value: &str) -> Result<Self, BridgeError> {
        match value {
            "init" => Ok(Self::Init),
            "status" => Ok(Self::Status),
            "shutdown" => Ok(Self::Shutdown),
            other => Err(BridgeError::UnknownMethod(other.to_string())),
        }
    }

    pub fn dead_name(self) -> &'static str {
        "dead-method"
    }
}

pub struct WireFrame {
    method: String,
    session_id: String,
    payload: String,
}

impl WireFrame {
    pub fn decode(raw: &str) -> Result<Self, BridgeError> {
        let mut parts = raw.splitn(3, ':');
        let method = parts.next().ok_or(BridgeError::InvalidFrame)?;
        let session_id = parts.next().ok_or(BridgeError::InvalidFrame)?;
        let payload = parts.next().ok_or(BridgeError::InvalidFrame)?;
        Ok(Self {
            method: method.to_string(),
            session_id: session_id.to_string(),
            payload: payload.to_string(),
        })
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn dead_wire_debug(&self) -> String {
        format!("dead-wire:{}", self.method)
    }
}

pub struct ResponseFrame {
    pub session_id: String,
    pub body: String,
}

impl ResponseFrame {
    pub fn ok(session_id: &str, body: String) -> Self {
        Self {
            session_id: session_id.to_string(),
            body,
        }
    }

    pub fn dead_response() -> Self {
        Self {
            session_id: "dead".to_string(),
            body: "dead".to_string(),
        }
    }
}

pub enum BridgeError {
    InvalidFrame,
    UnknownMethod(String),
    Core(String),
    Dead(String),
}

impl BridgeError {
    pub fn core(message: &str) -> Self {
        Self::Core(message.to_string())
    }

    pub fn dead(message: &str) -> Self {
        Self::Dead(message.to_string())
    }
}

pub fn dead_live_protocol() -> ResponseFrame {
    ResponseFrame::dead_response()
}

