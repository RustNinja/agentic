use runtime_support::{shared_runtime, RuntimeCore, RuntimeSnapshot};
use std::sync::Arc;

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct RuntimeBridge {
    runtime: Arc<RuntimeCore>,
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct RuntimeStatusDto {
    pub label: String,
    pub ready: bool,
}

impl RuntimeStatusDto {
    pub fn from_snapshot(snapshot: RuntimeSnapshot) -> Self {
        Self {
            label: snapshot.label().to_string(),
            ready: snapshot.ready(),
        }
    }

    pub fn render(&self) -> String {
        format!("{}:{}", self.label, self.ready)
    }

    pub fn dead_render(&self) -> String {
        format!("dead-status:{}", self.label)
    }
}

#[cfg_attr(feature = "ffi", uniffi::export(async_runtime = "tokio"))]
impl RuntimeBridge {
    pub fn shared() -> Self {
        Self {
            runtime: shared_runtime(),
        }
    }

    pub async fn current_status(&self, raw: &str) -> RuntimeStatusDto {
        RuntimeStatusDto::from_snapshot(self.runtime.status(raw).await)
    }

    pub async fn dead_exported_status(&self, raw: &str) -> String {
        self.runtime.dead_status(raw).await
    }
}

impl RuntimeBridge {
    pub fn dead_local_bridge(raw: &str) -> String {
        format!("dead-bridge:{raw}")
    }
}

pub async fn selected_async_runtime_status(raw: &str) -> String {
    RuntimeBridge::shared().current_status(raw).await.render()
}

pub async fn dead_live_async_runtime_status(raw: &str) -> String {
    RuntimeBridge::dead_local_bridge(raw)
}
