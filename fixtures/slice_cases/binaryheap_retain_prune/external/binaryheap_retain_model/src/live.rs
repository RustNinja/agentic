use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryHeapRetainItem {
    value: String,
}

impl BinaryHeapRetainItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn other() -> Self {
        Self::new("other")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value.contains("live")
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-live");
        self
    }

    pub fn sort_key(&self) -> usize {
        self.value.len()
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-retain:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-binaryheap-retain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-retain:{}", self.value)
    }
}

fn binaryheap_retain_entries(raw: &str) -> BinaryHeap<BinaryHeapRetainItem> {
    let mut heap = BinaryHeap::new();
    heap.push(BinaryHeapRetainItem::live());
    heap.push(BinaryHeapRetainItem::other());
    heap.push(BinaryHeapRetainItem::new(raw));
    heap
}

pub fn selected_binaryheap_retain(raw: &str) -> String {
    let mut entries = binaryheap_retain_entries(raw);
    entries.retain(|item| item.is_live());
    entries
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_binaryheap_retain(raw: &str) -> String {
    BinaryHeapRetainItem::new(raw).dead_method()
}
