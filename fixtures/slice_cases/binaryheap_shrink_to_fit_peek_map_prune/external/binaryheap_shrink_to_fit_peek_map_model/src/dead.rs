pub struct DeadBinaryheapShrinkToFitPeekMapItem {
    value: String,
}

impl DeadBinaryheapShrinkToFitPeekMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-shrink-to-fit-peek-map:{}", self.value)
    }
}

pub fn dead_binaryheap_shrink_to_fit_peek_map(raw: &str) -> String {
    DeadBinaryheapShrinkToFitPeekMapItem::new(raw).dead_method()
}
