use std::sync::Mutex;

static LIVE_REGISTRY: Mutex<Vec<MutexEntry>> = Mutex::new(Vec::new());

pub struct MutexEntry {
    label: String,
}

impl MutexEntry {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("mutex:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mutex:{}", self.label)
    }
}

pub fn selected_mutex(raw: &str) -> String {
    let mut guard = LIVE_REGISTRY.lock().expect("mutex registry should lock");
    guard.push(MutexEntry::new(raw));
    guard.last().map(MutexEntry::render).unwrap_or_default()
}

pub fn dead_live_mutex(raw: &str) -> String {
    MutexEntry::new(raw).dead_method()
}
