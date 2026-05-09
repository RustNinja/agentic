use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapIntoValuesNextKey {
    value: &'static str,
}

impl HashmapIntoValuesNextKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct HashmapIntoValuesNextPayload {
    value: String,
    touched: bool,
}

impl HashmapIntoValuesNextPayload {
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
        format!("hashmap-into-values-next:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-into-values-next:{}", self.value)
    }
}

fn hashmap_into_values_next_entries(raw: &str) -> HashMap<HashmapIntoValuesNextKey, HashmapIntoValuesNextPayload> {
    let mut entries = HashMap::new();
    entries.insert(HashmapIntoValuesNextKey::live(), HashmapIntoValuesNextPayload::new(raw));
    entries.insert(HashmapIntoValuesNextKey::tail(), HashmapIntoValuesNextPayload::new("tail"));
    entries
}

pub fn selected_hashmap_into_values_next(raw: &str) -> String {
    let entries = hashmap_into_values_next_entries(raw);
    entries
        .into_values()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "hashmap-into-values-next:missing".to_string())
}

pub fn dead_live_hashmap_into_values_next(raw: &str) -> String {
    HashmapIntoValuesNextPayload::new(raw).dead_method()
}
