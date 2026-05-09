use std::collections::BinaryHeap;
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BinaryheapIntoSortedVecItem {
    value: String,
}

impl BinaryheapIntoSortedVecItem {
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
        format!("binaryheap-into-sorted-vec:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-into-sorted-vec:{}", self.value)
    }
}

fn binaryheap_into_sorted_vec_entries(raw: &str) -> BinaryHeap<BinaryheapIntoSortedVecItem> {
    let mut entries = BinaryHeap::new();
    entries.push(BinaryheapIntoSortedVecItem::live());
    entries.push(BinaryheapIntoSortedVecItem::new(raw));
    entries
}

pub fn selected_binaryheap_into_sorted_vec(raw: &str) -> String {
    let entries = binaryheap_into_sorted_vec_entries(raw);
    entries
        .into_sorted_vec()
        .into_iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_binaryheap_into_sorted_vec(raw: &str) -> String {
    BinaryheapIntoSortedVecItem::new(raw).dead_method()
}
