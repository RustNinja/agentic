use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreesetPopFirstItem {
    value: String,
}

impl BtreesetPopFirstItem {
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
        format!("btreeset-pop-first:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-pop-first:{}", self.value)
    }
}

fn btreeset_pop_first_entries(raw: &str) -> BTreeSet<BtreesetPopFirstItem> {
    let mut entries = BTreeSet::new();
    entries.insert(BtreesetPopFirstItem::live());
    entries.insert(BtreesetPopFirstItem::new(raw));
    entries
}

pub fn selected_btreeset_pop_first(raw: &str) -> String {
    let mut entries = btreeset_pop_first_entries(raw);
    entries
        .pop_first()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "btreeset-pop-first:missing".to_string())
}

pub fn dead_live_btreeset_pop_first(raw: &str) -> String {
    BtreesetPopFirstItem::new(raw).dead_method()
}
