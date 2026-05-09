use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapIterMutPairsKey {
    value: &'static str,
}

impl HashmapIterMutPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("hashmap-iter-mut-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-mut-pairs-key:{}", self.value)
    }
}

pub struct HashmapIterMutPairsPayload {
    value: String,
    touched: bool,
}

impl HashmapIterMutPairsPayload {
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
        format!("hashmap-iter-mut-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-iter-mut-pairs:{}", self.value)
    }
}

fn hashmap_iter_mut_pairs_entries(
    raw: &str,
) -> HashMap<HashmapIterMutPairsKey, HashmapIterMutPairsPayload> {
    let mut entries = HashMap::new();
    entries.insert(
        HashmapIterMutPairsKey::live(),
        HashmapIterMutPairsPayload::new(raw),
    );
    entries
}

pub fn selected_hashmap_iter_mut_pairs(raw: &str) -> String {
    let mut entries = hashmap_iter_mut_pairs_entries(raw);
    entries.iter_mut().for_each(|(_key, payload)| {
        payload.mark_live();
    });
    entries
        .values()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "hashmap-iter-mut-pairs:missing".to_string())
}

pub fn dead_live_hashmap_iter_mut_pairs(raw: &str) -> String {
    HashmapIterMutPairsKey::live().dead_method()
        + &HashmapIterMutPairsPayload::new(raw).dead_method()
}
