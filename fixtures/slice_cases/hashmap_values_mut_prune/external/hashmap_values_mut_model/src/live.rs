use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct HashmapValuesMutKey {
    value: String,
}

impl HashmapValuesMutKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-hashmap-values-mut-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct HashmapValuesMutPayload {
    value: String,
}

impl HashmapValuesMutPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-values-mut:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("hashmap-values-mut:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-values-mut:{}", self.value)
    }
}

fn hashmap_values_mut_map(raw: &str) -> HashMap<HashmapValuesMutKey, HashmapValuesMutPayload> {
    let mut payloads = HashMap::new();
    payloads.insert(
        HashmapValuesMutKey::new(raw),
        HashmapValuesMutPayload::new(raw),
    );
    payloads.insert(
        HashmapValuesMutKey::new("tail"),
        HashmapValuesMutPayload::new("tail"),
    );
    payloads
}

pub fn selected_hashmap_values_mut(raw: &str) -> String {
    let mut payloads = hashmap_values_mut_map(raw);
    payloads
        .values_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "hashmap-values-mut:missing".to_string())
}

pub fn dead_live_hashmap_values_mut(raw: &str) -> String {
    let key = HashmapValuesMutKey::new(raw);
    let payload = HashmapValuesMutPayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
