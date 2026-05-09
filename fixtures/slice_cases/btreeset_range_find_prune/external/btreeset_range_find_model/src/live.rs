use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreesetRangeFindItem {
    value: String,
}

impl BtreesetRangeFindItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-range-find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-range-find:{}", self.value)
    }
}

fn btreeset_range_find_entries(raw: &str) -> BTreeSet<BtreesetRangeFindItem> {
    let mut entries = BTreeSet::new();
    entries.insert(BtreesetRangeFindItem::live());
    entries.insert(BtreesetRangeFindItem::new(raw));
    entries
}

pub fn selected_btreeset_range_find(raw: &str) -> String {
    let entries = btreeset_range_find_entries(raw);
    entries
        .range(BtreesetRangeFindItem::live()..)
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "btreeset-range-find:missing".to_string())
}

pub fn dead_live_btreeset_range_find(raw: &str) -> String {
    BtreesetRangeFindItem::new(raw).dead_method()
}
