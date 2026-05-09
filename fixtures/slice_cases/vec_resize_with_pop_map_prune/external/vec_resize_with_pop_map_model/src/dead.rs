pub struct DeadVecResizeWithPopMapItem {
    value: String,
}

impl DeadVecResizeWithPopMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-resize-with-pop-map:{}", self.value)
    }
}

pub fn dead_vec_resize_with_pop_map(raw: &str) -> String {
    DeadVecResizeWithPopMapItem::new(raw).dead_method()
}
