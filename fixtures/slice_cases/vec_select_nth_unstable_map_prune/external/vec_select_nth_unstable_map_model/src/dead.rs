pub struct DeadVecSelectNthUnstableMapItem {
    value: String,
}

impl DeadVecSelectNthUnstableMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-select-nth-unstable-map:{}", self.value)
    }
}

pub fn dead_vec_select_nth_unstable_map(raw: &str) -> String {
    DeadVecSelectNthUnstableMapItem::new(raw).dead_method()
}
