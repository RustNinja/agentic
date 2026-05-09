use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapIterMutPairsKey {
    value: &'static str,
}

impl BtreemapIterMutPairsKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-iter-mut-pairs-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-iter-mut-pairs-key:{}", self.value)
    }
}

pub struct BtreemapIterMutPairsPayload {
    value: String,
    touched: bool,
}

impl BtreemapIterMutPairsPayload {
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
        format!("btreemap-iter-mut-pairs:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-iter-mut-pairs:{}", self.value)
    }
}

fn btreemap_iter_mut_pairs_entries(
    raw: &str,
) -> BTreeMap<BtreemapIterMutPairsKey, BtreemapIterMutPairsPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(
        BtreemapIterMutPairsKey::live(),
        BtreemapIterMutPairsPayload::new(raw),
    );
    entries
}

pub fn selected_btreemap_iter_mut_pairs(raw: &str) -> String {
    let mut entries = btreemap_iter_mut_pairs_entries(raw);
    entries.iter_mut().for_each(|(_key, payload)| {
        payload.mark_live();
    });
    entries
        .values()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-iter-mut-pairs:missing".to_string())
}

pub fn dead_live_btreemap_iter_mut_pairs(raw: &str) -> String {
    BtreemapIterMutPairsKey::live().dead_method()
        + &BtreemapIterMutPairsPayload::new(raw).dead_method()
}
