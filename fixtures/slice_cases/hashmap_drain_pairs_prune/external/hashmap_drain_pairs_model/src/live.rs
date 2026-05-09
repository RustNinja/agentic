use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapDrainPairsKey {
    value: &'static str,
}

impl HashmapDrainPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("hashmap-drain-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-drain-pairs-key:{}", self.value)
    }
}

pub struct HashmapDrainPairsPayload {
    value: String,
    touched: bool,
}

impl HashmapDrainPairsPayload {
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
        format!("hashmap-drain-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-drain-pairs:{}", self.value)
    }
}

fn hashmap_drain_pairs_entries(
    raw: &str,
) -> HashMap<HashmapDrainPairsKey, HashmapDrainPairsPayload> {
    let mut entries = HashMap::new();
    entries.insert(
        HashmapDrainPairsKey::live(),
        HashmapDrainPairsPayload::new(raw),
    );
    entries
}

pub fn selected_hashmap_drain_pairs(raw: &str) -> String {
    let mut entries = hashmap_drain_pairs_entries(raw);
    entries
        .drain()
        .map(|(_key, payload)| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashmap_drain_pairs(raw: &str) -> String {
    HashmapDrainPairsKey::live().dead_method() + &HashmapDrainPairsPayload::new(raw).dead_method()
}
