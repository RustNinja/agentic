use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreesetGetItem {
    value: String,
}

impl BtreesetGetItem {
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
        format!("btreeset-get:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-get:{}", self.value)
    }
}

fn btreeset_get_entries(raw: &str) -> BTreeSet<BtreesetGetItem> {
    let mut entries = BTreeSet::new();
    entries.insert(BtreesetGetItem::live());
    entries.insert(BtreesetGetItem::new(raw));
    entries
}

pub fn selected_btreeset_get(raw: &str) -> String {
    let entries = btreeset_get_entries(raw);
    entries
        .get(&BtreesetGetItem::live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "btreeset-get:missing".to_string())
}

pub fn dead_live_btreeset_get(raw: &str) -> String {
    BtreesetGetItem::new(raw).dead_method()
}
