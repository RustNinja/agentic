use std::collections::BinaryHeap;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryheapPopItem {
    value: String,
}

impl BinaryheapPopItem {
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
        format!("binaryheap-pop:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-pop:{}", self.value)
    }
}

fn binaryheap_pop_entries(raw: &str) -> BinaryHeap<BinaryheapPopItem> {
    let mut entries = BinaryHeap::new();
    entries.push(BinaryheapPopItem::live());
    entries.push(BinaryheapPopItem::new(raw));
    entries
}

pub fn selected_binaryheap_pop(raw: &str) -> String {
    let mut entries = binaryheap_pop_entries(raw);
    entries
        .pop()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "binaryheap-pop:missing".to_string())
}

pub fn dead_live_binaryheap_pop(raw: &str) -> String {
    BinaryheapPopItem::new(raw).dead_method()
}
