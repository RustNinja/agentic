use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapRangeMutPairsKey {
    value: &'static str,
}

impl BtreemapRangeMutPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-range-mut-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-mut-pairs-key:{}", self.value)
    }
}

pub struct BtreemapRangeMutPairsPayload {
    value: String,
    touched: bool,
}

impl BtreemapRangeMutPairsPayload {
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
        format!("btreemap-range-mut-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-mut-pairs:{}", self.value)
    }
}

fn btreemap_range_mut_pairs_entries(
    raw: &str,
) -> BTreeMap<BtreemapRangeMutPairsKey, BtreemapRangeMutPairsPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(
        BtreemapRangeMutPairsKey::live(),
        BtreemapRangeMutPairsPayload::new(raw),
    );
    entries
}

pub fn selected_btreemap_range_mut_pairs(raw: &str) -> String {
    let mut entries = btreemap_range_mut_pairs_entries(raw);
    entries
        .range_mut(BtreemapRangeMutPairsKey::live()..)
        .for_each(|(_key, payload)| {
            payload.mark_live();
        });
    entries
        .values()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-range-mut-pairs:missing".to_string())
}

pub fn dead_live_btreemap_range_mut_pairs(raw: &str) -> String {
    BtreemapRangeMutPairsKey::live().dead_method()
        + &BtreemapRangeMutPairsPayload::new(raw).dead_method()
}
