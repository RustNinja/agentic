use std::sync::{Arc, OnceLock};

static RUNTIME: OnceLock<Arc<RuntimeObject>> = OnceLock::new();

pub struct RuntimeObject {
    prefix: String,
}

impl RuntimeObject {
    pub fn new() -> Self {
        Self {
            prefix: "runtime".to_string(),
        }
    }

    pub fn status(&self, raw: &str) -> String {
        format!("{}:{}", self.prefix, raw.trim())
    }

    pub fn dead_method(&self, raw: &str) -> String {
        format!("dead-runtime:{raw}")
    }
}

pub fn shared_runtime() -> Arc<RuntimeObject> {
    RUNTIME.get_or_init(|| Arc::new(RuntimeObject::new())).clone()
}

pub fn selected_runtime(raw: &str) -> String {
    shared_runtime().status(raw)
}

pub fn dead_live_runtime(raw: &str) -> String {
    RuntimeObject::new().dead_method(raw)
}
