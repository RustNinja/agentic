use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapGetKey {
    value: &'static str,
}

impl HashmapGetKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct HashmapGetPayload {
    value: String,
    touched: bool,
}

impl HashmapGetPayload {
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
        format!("hashmap-get:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-get:{}", self.value)
    }
}

fn hashmap_get_entries(raw: &str) -> HashMap<HashmapGetKey, HashmapGetPayload> {
    let mut entries = HashMap::new();
    entries.insert(HashmapGetKey::live(), HashmapGetPayload::new(raw));
    entries.insert(HashmapGetKey::tail(), HashmapGetPayload::new("tail"));
    entries
}

pub fn selected_hashmap_get(raw: &str) -> String {
    let entries = hashmap_get_entries(raw);
    entries
        .get(&HashmapGetKey::live())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "hashmap-get:missing".to_string())
}

pub fn dead_live_hashmap_get(raw: &str) -> String {
    HashmapGetPayload::new(raw).dead_method()
}
