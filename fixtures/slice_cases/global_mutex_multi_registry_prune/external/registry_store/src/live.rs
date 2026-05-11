use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

static REGISTRY: OnceLock<Mutex<BTreeMap<String, Arc<RegistryEntry>>>> = OnceLock::new();

#[derive(Clone)]
pub struct RegistryEntry {
    key: String,
    value: String,
}

impl RegistryEntry {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("{}={}", self.key, self.value)
    }

    pub fn dead_render(&self) -> String {
        format!("dead-registry-entry:{}={}", self.key, self.value)
    }
}

fn registry() -> &'static Mutex<BTreeMap<String, Arc<RegistryEntry>>> {
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub fn register_entry(key: &str, value: &str) {
    let entry = Arc::new(RegistryEntry::new(key, value));
    let mut guard = registry().lock().expect("registry lock should not be poisoned");
    guard.insert(key.to_string(), entry);
}

pub fn lookup_entry(key: &str) -> Option<Arc<RegistryEntry>> {
    let guard = registry().lock().expect("registry lock should not be poisoned");
    guard.get(key).cloned()
}

pub fn remove_entry(key: &str) -> Option<Arc<RegistryEntry>> {
    let mut guard = registry().lock().expect("registry lock should not be poisoned");
    guard.remove(key)
}

pub fn list_entries() -> Vec<Arc<RegistryEntry>> {
    let guard = registry().lock().expect("registry lock should not be poisoned");
    guard.values().cloned().collect()
}

pub fn dead_live_registry(raw: &str) -> String {
    register_entry("dead", raw);
    list_entries()
        .into_iter()
        .map(|entry| entry.dead_render())
        .collect::<Vec<_>>()
        .join(",")
}
