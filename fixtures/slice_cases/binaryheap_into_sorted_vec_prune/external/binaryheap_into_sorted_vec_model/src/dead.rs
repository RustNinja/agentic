pub struct DeadBinaryheapIntoSortedVecItem {
    value: String,
}

impl DeadBinaryheapIntoSortedVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-into-sorted-vec:{}", self.value)
    }
}

pub fn dead_binaryheap_into_sorted_vec(raw: &str) -> String {
    DeadBinaryheapIntoSortedVecItem::new(raw).dead_method()
}
