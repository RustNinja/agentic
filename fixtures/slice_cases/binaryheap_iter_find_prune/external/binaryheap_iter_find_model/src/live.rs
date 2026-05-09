use std::collections::BinaryHeap;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryheapIterFindItem {
    value: String,
}

impl BinaryheapIterFindItem {
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
        format!("binaryheap-iter-find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-iter-find:{}", self.value)
    }
}

fn binaryheap_iter_find_entries(raw: &str) -> BinaryHeap<BinaryheapIterFindItem> {
    let mut entries = BinaryHeap::new();
    entries.push(BinaryheapIterFindItem::live());
    entries.push(BinaryheapIterFindItem::new(raw));
    entries
}

pub fn selected_binaryheap_iter_find(raw: &str) -> String {
    let entries = binaryheap_iter_find_entries(raw);
    entries
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "binaryheap-iter-find:missing".to_string())
}

pub fn dead_live_binaryheap_iter_find(raw: &str) -> String {
    BinaryheapIterFindItem::new(raw).dead_method()
}
