use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapRemoveKey {
    value: &'static str,
}

impl HashmapRemoveKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct HashmapRemovePayload {
    value: String,
    touched: bool,
}

impl HashmapRemovePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
            touched: false,
        }
    }

    pub fn mark_live(&mut self) -> &mut Self {
        self.touched = true;
        self
    }

    pub fn render_label(&self) -> String {
        let suffix = if self.touched { ":touched" } else { "" };
        format!("hashmap-remove:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-remove:{}", self.value)
    }
}

fn hashmap_remove_entries(raw: &str) -> HashMap<HashmapRemoveKey, HashmapRemovePayload> {
    let mut entries = HashMap::new();
    entries.insert(HashmapRemoveKey::live(), HashmapRemovePayload::new(raw));
    entries.insert(HashmapRemoveKey::tail(), HashmapRemovePayload::new("tail"));
    entries
}

pub fn selected_hashmap_remove(raw: &str) -> String {
    let mut entries = hashmap_remove_entries(raw);
    entries
        .remove(&HashmapRemoveKey::live())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "hashmap-remove:missing".to_string())
}

pub fn dead_live_hashmap_remove(raw: &str) -> String {
    HashmapRemovePayload::new(raw).dead_method()
}
