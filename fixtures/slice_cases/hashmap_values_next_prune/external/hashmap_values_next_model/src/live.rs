use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapValuesNextKey {
    value: &'static str,
}

impl HashmapValuesNextKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct HashmapValuesNextPayload {
    value: String,
    touched: bool,
}

impl HashmapValuesNextPayload {
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
        format!("hashmap-values-next:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-values-next:{}", self.value)
    }
}

fn hashmap_values_next_entries(raw: &str) -> HashMap<HashmapValuesNextKey, HashmapValuesNextPayload> {
    let mut entries = HashMap::new();
    entries.insert(HashmapValuesNextKey::live(), HashmapValuesNextPayload::new(raw));
    entries.insert(HashmapValuesNextKey::tail(), HashmapValuesNextPayload::new("tail"));
    entries
}

pub fn selected_hashmap_values_next(raw: &str) -> String {
    let entries = hashmap_values_next_entries(raw);
    entries
        .values()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "hashmap-values-next:missing".to_string())
}

pub fn dead_live_hashmap_values_next(raw: &str) -> String {
    HashmapValuesNextPayload::new(raw).dead_method()
}
