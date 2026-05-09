use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapIntoIterPairsKey {
    value: &'static str,
}

impl BtreemapIntoIterPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-into-iter-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-into-iter-pairs-key:{}", self.value)
    }
}

pub struct BtreemapIntoIterPairsPayload {
    value: String,
    touched: bool,
}

impl BtreemapIntoIterPairsPayload {
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
        format!("btreemap-into-iter-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-into-iter-pairs:{}", self.value)
    }
}

fn btreemap_into_iter_pairs_entries(
    raw: &str,
) -> BTreeMap<BtreemapIntoIterPairsKey, BtreemapIntoIterPairsPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(
        BtreemapIntoIterPairsKey::live(),
        BtreemapIntoIterPairsPayload::new(raw),
    );
    entries
}

pub fn selected_btreemap_into_iter_pairs(raw: &str) -> String {
    let entries = btreemap_into_iter_pairs_entries(raw);
    entries
        .into_iter()
        .map(|(_key, payload)| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_btreemap_into_iter_pairs(raw: &str) -> String {
    BtreemapIntoIterPairsKey::live().dead_method()
        + &BtreemapIntoIterPairsPayload::new(raw).dead_method()
}
