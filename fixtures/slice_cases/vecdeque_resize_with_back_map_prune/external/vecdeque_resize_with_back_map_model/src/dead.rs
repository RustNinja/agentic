pub struct DeadVecdequeResizeWithBackMapItem {
    value: String,
}

impl DeadVecdequeResizeWithBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-resize-with-back-map:{}", self.value)
    }
}

pub fn dead_vecdeque_resize_with_back_map(raw: &str) -> String {
    DeadVecdequeResizeWithBackMapItem::new(raw).dead_method()
}
