use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreesetPopLastItem {
    value: String,
}

impl BtreesetPopLastItem {
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
        format!("btreeset-pop-last:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-pop-last:{}", self.value)
    }
}

fn btreeset_pop_last_entries(raw: &str) -> BTreeSet<BtreesetPopLastItem> {
    let mut entries = BTreeSet::new();
    entries.insert(BtreesetPopLastItem::live());
    entries.insert(BtreesetPopLastItem::new(raw));
    entries
}

pub fn selected_btreeset_pop_last(raw: &str) -> String {
    let mut entries = btreeset_pop_last_entries(raw);
    entries
        .pop_last()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "btreeset-pop-last:missing".to_string())
}

pub fn dead_live_btreeset_pop_last(raw: &str) -> String {
    BtreesetPopLastItem::new(raw).dead_method()
}
