use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreesetReplaceItem {
    value: String,
}

impl BtreesetReplaceItem {
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
        format!("btreeset-replace:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-replace:{}", self.value)
    }
}

fn btreeset_replace_entries(raw: &str) -> BTreeSet<BtreesetReplaceItem> {
    let mut entries = BTreeSet::new();
    entries.insert(BtreesetReplaceItem::live());
    entries.insert(BtreesetReplaceItem::new(raw));
    entries
}

pub fn selected_btreeset_replace(raw: &str) -> String {
    let mut entries = btreeset_replace_entries(raw);
    entries
        .replace(BtreesetReplaceItem::live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "btreeset-replace:missing".to_string())
}

pub fn dead_live_btreeset_replace(raw: &str) -> String {
    BtreesetReplaceItem::new(raw).dead_method()
}
