pub struct DeadBinaryheapIntoSortedVecMapItem {
    value: String,
}

impl DeadBinaryheapIntoSortedVecMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-into-sorted-vec-map:{}", self.value)
    }
}

pub fn dead_binaryheap_into_sorted_vec_map(raw: &str) -> String {
    DeadBinaryheapIntoSortedVecMapItem::new(raw).dead_method()
}
