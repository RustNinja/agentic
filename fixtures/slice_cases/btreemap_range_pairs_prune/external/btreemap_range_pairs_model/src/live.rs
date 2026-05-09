use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapRangePairsKey {
    value: &'static str,
}

impl BtreemapRangePairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-range-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-pairs-key:{}", self.value)
    }
}

pub struct BtreemapRangePairsPayload {
    value: String,
    touched: bool,
}

impl BtreemapRangePairsPayload {
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
        format!("btreemap-range-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-pairs:{}", self.value)
    }
}

fn btreemap_range_pairs_entries(
    raw: &str,
) -> BTreeMap<BtreemapRangePairsKey, BtreemapRangePairsPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(
        BtreemapRangePairsKey::live(),
        BtreemapRangePairsPayload::new(raw),
    );
    entries
}

pub fn selected_btreemap_range_pairs(raw: &str) -> String {
    let entries = btreemap_range_pairs_entries(raw);
    entries
        .range(BtreemapRangePairsKey::live()..)
        .map(|(_key, payload)| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_btreemap_range_pairs(raw: &str) -> String {
    BtreemapRangePairsKey::live().dead_method() + &BtreemapRangePairsPayload::new(raw).dead_method()
}
