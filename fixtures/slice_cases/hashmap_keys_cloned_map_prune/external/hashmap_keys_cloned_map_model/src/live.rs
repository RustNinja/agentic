use std::collections::HashMap;
pub struct HashmapKeysClonedMapPayload {
    value: String,
}

impl HashmapKeysClonedMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-keys-cloned-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-keys-cloned-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-keys-cloned-map:{}", self.value)
    }
}

pub fn selected_hashmap_keys_cloned_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert(raw.to_string(), 1usize);
    values
        .keys()
        .cloned()
        .map(|key| HashmapKeysClonedMapPayload::new(&key).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_hashmap_keys_cloned_map(raw: &str) -> String {
    HashmapKeysClonedMapPayload::new(raw).unused_label()
}
