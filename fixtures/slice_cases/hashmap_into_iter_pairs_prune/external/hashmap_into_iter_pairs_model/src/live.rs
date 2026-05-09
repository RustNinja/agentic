use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapIntoIterPairsKey {
    value: &'static str,
}

impl HashmapIntoIterPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("hashmap-into-iter-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-into-iter-pairs-key:{}", self.value)
    }
}

pub struct HashmapIntoIterPairsPayload {
    value: String,
    touched: bool,
}

impl HashmapIntoIterPairsPayload {
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
        format!("hashmap-into-iter-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-into-iter-pairs:{}", self.value)
    }
}

fn hashmap_into_iter_pairs_entries(
    raw: &str,
) -> HashMap<HashmapIntoIterPairsKey, HashmapIntoIterPairsPayload> {
    let mut entries = HashMap::new();
    entries.insert(
        HashmapIntoIterPairsKey::live(),
        HashmapIntoIterPairsPayload::new(raw),
    );
    entries
}

pub fn selected_hashmap_into_iter_pairs(raw: &str) -> String {
    let entries = hashmap_into_iter_pairs_entries(raw);
    entries
        .into_iter()
        .map(|(_key, payload)| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashmap_into_iter_pairs(raw: &str) -> String {
    HashmapIntoIterPairsKey::live().dead_method()
        + &HashmapIntoIterPairsPayload::new(raw).dead_method()
}
