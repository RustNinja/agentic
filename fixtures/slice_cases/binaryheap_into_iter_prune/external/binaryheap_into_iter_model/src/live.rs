use std::collections::BinaryHeap;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryheapIntoIterItem {
    value: String,
}

impl BinaryheapIntoIterItem {
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
        format!("binaryheap-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-into-iter:{}", self.value)
    }
}

fn binaryheap_into_iter_entries(raw: &str) -> BinaryHeap<BinaryheapIntoIterItem> {
    let mut entries = BinaryHeap::new();
    entries.push(BinaryheapIntoIterItem::live());
    entries.push(BinaryheapIntoIterItem::new(raw));
    entries
}

pub fn selected_binaryheap_into_iter(raw: &str) -> String {
    let entries = binaryheap_into_iter_entries(raw);
    entries
        .into_iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_binaryheap_into_iter(raw: &str) -> String {
    BinaryheapIntoIterItem::new(raw).dead_method()
}
