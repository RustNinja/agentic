use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapIterPairsKey {
    value: &'static str,
}

impl BtreemapIterPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-iter-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-iter-pairs-key:{}", self.value)
    }
}

pub struct BtreemapIterPairsPayload {
    value: String,
    touched: bool,
}

impl BtreemapIterPairsPayload {
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
        format!("btreemap-iter-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-iter-pairs:{}", self.value)
    }
}

fn btreemap_iter_pairs_entries(
    raw: &str,
) -> BTreeMap<BtreemapIterPairsKey, BtreemapIterPairsPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(
        BtreemapIterPairsKey::live(),
        BtreemapIterPairsPayload::new(raw),
    );
    entries
}

pub fn selected_btreemap_iter_pairs(raw: &str) -> String {
    let entries = btreemap_iter_pairs_entries(raw);
    entries
        .iter()
        .map(|(_key, payload)| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_btreemap_iter_pairs(raw: &str) -> String {
    BtreemapIterPairsKey::live().dead_method() + &BtreemapIterPairsPayload::new(raw).dead_method()
}
