use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreesetTakeItem {
    value: String,
}

impl BtreesetTakeItem {
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
        format!("btreeset-take:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-take:{}", self.value)
    }
}

fn btreeset_take_entries(raw: &str) -> BTreeSet<BtreesetTakeItem> {
    let mut entries = BTreeSet::new();
    entries.insert(BtreesetTakeItem::live());
    entries.insert(BtreesetTakeItem::new(raw));
    entries
}

pub fn selected_btreeset_take(raw: &str) -> String {
    let mut entries = btreeset_take_entries(raw);
    entries
        .take(&BtreesetTakeItem::live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "btreeset-take:missing".to_string())
}

pub fn dead_live_btreeset_take(raw: &str) -> String {
    BtreesetTakeItem::new(raw).dead_method()
}
