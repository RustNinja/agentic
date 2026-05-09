pub struct DeadArrayIntoIterSkipMapItem {
    value: String,
}

impl DeadArrayIntoIterSkipMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-array-into-iter-skip-map:{}", self.value)
    }
}

pub fn dead_array_into_iter_skip_map(raw: &str) -> String {
    DeadArrayIntoIterSkipMapItem::new(raw).render()
}
