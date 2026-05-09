use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapGetMutKey {
    value: &'static str,
}

impl HashmapGetMutKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct HashmapGetMutPayload {
    value: String,
    touched: bool,
}

impl HashmapGetMutPayload {
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
        format!("hashmap-get-mut:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-get-mut:{}", self.value)
    }
}

fn hashmap_get_mut_entries(raw: &str) -> HashMap<HashmapGetMutKey, HashmapGetMutPayload> {
    let mut entries = HashMap::new();
    entries.insert(HashmapGetMutKey::live(), HashmapGetMutPayload::new(raw));
    entries.insert(HashmapGetMutKey::tail(), HashmapGetMutPayload::new("tail"));
    entries
}

pub fn selected_hashmap_get_mut(raw: &str) -> String {
    let mut entries = hashmap_get_mut_entries(raw);
    entries
        .get_mut(&HashmapGetMutKey::live())
        .map(|payload| payload.mark_live().render_label())
        .unwrap_or_else(|| "hashmap-get-mut:missing".to_string())
}

pub fn dead_live_hashmap_get_mut(raw: &str) -> String {
    HashmapGetMutPayload::new(raw).dead_method()
}
