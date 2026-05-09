use std::collections::BinaryHeap;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryheapDrainItem {
    value: String,
}

impl BinaryheapDrainItem {
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
        format!("binaryheap-drain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-drain:{}", self.value)
    }
}

fn binaryheap_drain_entries(raw: &str) -> BinaryHeap<BinaryheapDrainItem> {
    let mut entries = BinaryHeap::new();
    entries.push(BinaryheapDrainItem::live());
    entries.push(BinaryheapDrainItem::new(raw));
    entries
}

pub fn selected_binaryheap_drain(raw: &str) -> String {
    let mut entries = binaryheap_drain_entries(raw);
    entries
        .drain()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_binaryheap_drain(raw: &str) -> String {
    BinaryheapDrainItem::new(raw).dead_method()
}
