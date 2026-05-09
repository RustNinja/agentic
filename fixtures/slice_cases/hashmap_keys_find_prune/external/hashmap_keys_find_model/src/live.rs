use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapKeysFindKey {
    value: &'static str,
}

impl HashmapKeysFindKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("hashmap-keys-find-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-keys-find-key:{}", self.value)
    }
}

pub struct HashmapKeysFindPayload {
    value: String,
    touched: bool,
}

impl HashmapKeysFindPayload {
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
        format!("hashmap-keys-find:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-keys-find:{}", self.value)
    }
}

fn hashmap_keys_find_entries(raw: &str) -> HashMap<HashmapKeysFindKey, HashmapKeysFindPayload> {
    let mut entries = HashMap::new();
    entries.insert(HashmapKeysFindKey::live(), HashmapKeysFindPayload::new(raw));
    entries
}

pub fn selected_hashmap_keys_find(raw: &str) -> String {
    let entries = hashmap_keys_find_entries(raw);
    entries
        .keys()
        .find(|key| key.is_live())
        .map(|key| key.render_key())
        .unwrap_or_else(|| "hashmap-keys-find:missing".to_string())
}

pub fn dead_live_hashmap_keys_find(raw: &str) -> String {
    HashmapKeysFindKey::live().dead_method() + &HashmapKeysFindPayload::new(raw).dead_method()
}
