use std::sync::{Mutex, OnceLock};

static DEAD_REGISTRY: OnceLock<Mutex<Option<DeadRegistry>>> = OnceLock::new();

pub struct DeadRegistry {
    label: String,
}

impl DeadRegistry {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-static:{}", self.label)
    }
}

pub fn dead_registry(raw: &str) -> String {
    let slot = DEAD_REGISTRY.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().expect("dead lock should not be poisoned");
    *guard = Some(DeadRegistry::new(raw));
    guard.take().expect("dead registry populated").render()
}
