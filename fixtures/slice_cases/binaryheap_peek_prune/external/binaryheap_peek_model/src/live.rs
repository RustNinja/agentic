use std::collections::BinaryHeap;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryheapPeekItem {
    value: String,
}

impl BinaryheapPeekItem {
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
        format!("binaryheap-peek:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-peek:{}", self.value)
    }
}

fn binaryheap_peek_entries(raw: &str) -> BinaryHeap<BinaryheapPeekItem> {
    let mut entries = BinaryHeap::new();
    entries.push(BinaryheapPeekItem::live());
    entries.push(BinaryheapPeekItem::new(raw));
    entries
}

pub fn selected_binaryheap_peek(raw: &str) -> String {
    let entries = binaryheap_peek_entries(raw);
    entries
        .peek()
        .map(|item| item.render_label())
        .unwrap_or_else(|| "binaryheap-peek:missing".to_string())
}

pub fn dead_live_binaryheap_peek(raw: &str) -> String {
    BinaryheapPeekItem::new(raw).dead_method()
}
