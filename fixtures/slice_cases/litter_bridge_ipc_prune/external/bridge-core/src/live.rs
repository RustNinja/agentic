use bridge_protocol::{BridgeError, Method};

#[derive(Clone, Copy)]
pub enum SessionState {
    Ready,
    Busy,
    Closed,
}

pub struct BridgeSession {
    id: String,
    state: SessionState,
}

impl BridgeSession {
    pub fn new(id: &str) -> Self {
        Self {
            id: normalize_id(id),
            state: SessionState::Ready,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn dead_debug(&self) -> String {
        format!("dead-session:{}", self.id)
    }
}

pub fn dispatch_method(
    session: &BridgeSession,
    method: Method,
    payload: &str,
) -> Result<String, BridgeError> {
    if payload.trim().is_empty() {
        return Err(BridgeError::core("empty bridge payload"));
    }

    match method {
        Method::Init => Ok(format!(
            "init:{}:{}",
            session.id(),
            normalize_payload(payload)
        )),
        Method::Status => Ok(format!(
            "status:{}:{}",
            session.id(),
            state_label(session.state())
        )),
        Method::Shutdown => Ok(format!("shutdown:{}", session.id())),
    }
}

fn normalize_id(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

fn normalize_payload(payload: &str) -> String {
    payload.trim().replace(' ', "_")
}

fn state_label(state: SessionState) -> &'static str {
    match state {
        SessionState::Ready => "ready",
        SessionState::Busy => "busy",
        SessionState::Closed => "closed",
    }
}

pub fn dead_live_dispatch(session: &BridgeSession) -> String {
    session.dead_debug()
}

