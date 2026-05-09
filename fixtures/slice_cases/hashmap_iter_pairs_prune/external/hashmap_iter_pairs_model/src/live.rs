use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapIterPairsKey {
    value: &'static str,
}

impl HashmapIterPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("hashmap-iter-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-pairs-key:{}", self.value)
    }
}

pub struct HashmapIterPairsPayload {
    value: String,
    touched: bool,
}

impl HashmapIterPairsPayload {
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
        format!("hashmap-iter-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-pairs:{}", self.value)
    }
}

fn hashmap_iter_pairs_entries(raw: &str) -> HashMap<HashmapIterPairsKey, HashmapIterPairsPayload> {
    let mut entries = HashMap::new();
    entries.insert(
        HashmapIterPairsKey::live(),
        HashmapIterPairsPayload::new(raw),
    );
    entries
}

pub fn selected_hashmap_iter_pairs(raw: &str) -> String {
    let entries = hashmap_iter_pairs_entries(raw);
    entries
        .iter()
        .map(|(_key, payload)| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashmap_iter_pairs(raw: &str) -> String {
    HashmapIterPairsKey::live().dead_method() + &HashmapIterPairsPayload::new(raw).dead_method()
}
