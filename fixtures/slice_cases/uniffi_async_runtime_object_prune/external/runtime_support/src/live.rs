use std::sync::{Arc, OnceLock};

static SHARED_RUNTIME: OnceLock<Arc<RuntimeCore>> = OnceLock::new();

pub struct RuntimeCore {
    name: String,
}

impl RuntimeCore {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub async fn status(&self, raw: &str) -> RuntimeSnapshot {
        RuntimeSnapshot {
            label: format!("{}:{}", self.name, raw.trim()),
            ready: true,
        }
    }

    pub async fn dead_status(&self, raw: &str) -> String {
        format!("dead-runtime-status:{}:{raw}", self.name)
    }
}

pub struct RuntimeSnapshot {
    label: String,
    ready: bool,
}

impl RuntimeSnapshot {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn ready(&self) -> bool {
        self.ready
    }

    pub fn dead_snapshot(&self) -> String {
        format!("dead-snapshot:{}", self.label)
    }
}

pub fn shared_runtime() -> Arc<RuntimeCore> {
    SHARED_RUNTIME
        .get_or_init(|| Arc::new(RuntimeCore::new("mobile-runtime")))
        .clone()
}

pub async fn dead_live_runtime_report(raw: &str) -> String {
    RuntimeCore::new("dead").dead_status(raw).await
}
