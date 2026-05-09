pub struct DeadBinaryheapAppendIntoSortedVecItem {
    value: String,
}

impl DeadBinaryheapAppendIntoSortedVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-binaryheap-append-into-sorted-vec:{}", self.value)
    }
}

pub fn dead_binaryheap_append_into_sorted_vec(raw: &str) -> String {
    DeadBinaryheapAppendIntoSortedVecItem::new(raw).render()
}
