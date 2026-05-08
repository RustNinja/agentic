use std::sync::{Arc, Mutex, OnceLock};

static LIVE_REGISTRY: OnceLock<Mutex<Option<Arc<RegistryHandle>>>> = OnceLock::new();

pub struct RegistryHandle {
    label: String,
}

impl RegistryHandle {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("registry:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-registry:{}", self.label)
    }
}

fn live_slot() -> &'static Mutex<Option<Arc<RegistryHandle>>> {
    LIVE_REGISTRY.get_or_init(|| Mutex::new(None))
}

pub fn selected_registry(raw: &str) -> String {
    let handle = Arc::new(RegistryHandle::new(raw));
    let mut guard = live_slot().lock().expect("registry lock should not be poisoned");
    *guard = Some(handle.clone());
    handle.render()
}

pub fn dead_live_registry(raw: &str) -> String {
    RegistryHandle::new(raw).dead_method()
}
