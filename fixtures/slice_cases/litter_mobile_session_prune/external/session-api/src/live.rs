use session_core::{SessionEngine, SessionStatus};
use session_protocol::{WireSession, WireStatusKind};

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct SessionRequest {
    label: String,
}

impl SessionRequest {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.trim().to_string(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct SessionHandle {
    engine: SessionEngine,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
impl SessionHandle {
    pub fn connect(request: SessionRequest) -> Self {
        Self {
            engine: SessionEngine::connect(request.label()),
        }
    }

    pub fn status(&self) -> SessionStatusDto {
        SessionStatusDto::from_status(self.engine.status())
    }

    pub fn dead_exported_status(&self) -> String {
        self.engine.dead_debug()
    }
}

impl SessionHandle {
    pub fn dead_connect(label: &str) -> DeadSessionApi {
        DeadSessionApi {
            label: label.to_string(),
        }
    }
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct SessionStatusDto {
    pub label: String,
    pub connected: bool,
}

impl SessionStatusDto {
    pub fn from_status(status: SessionStatus) -> Self {
        let wire = WireSession::from_status(status.label(), status.kind());
        Self {
            label: wire.label().to_string(),
            connected: matches!(wire.kind(), WireStatusKind::Connected),
        }
    }

    pub fn dead_render(&self) -> String {
        format!("dead-dto:{}", self.label)
    }
}

pub struct DeadSessionApi {
    label: String,
}

impl DeadSessionApi {
    pub fn dead_summary(self) -> String {
        format!("dead-session-api:{}", self.label)
    }
}

pub fn dead_live_api(label: &str) -> String {
    DeadSessionApi {
        label: label.to_string(),
    }
    .dead_summary()
}

