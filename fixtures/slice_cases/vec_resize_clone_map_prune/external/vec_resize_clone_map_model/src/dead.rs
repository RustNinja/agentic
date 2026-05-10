pub struct DeadVecResizeCloneMapItem {
    value: String,
}

impl DeadVecResizeCloneMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-resize-clone-map:{}", self.value)
    }
}

pub fn dead_vec_resize_clone_map(raw: &str) -> String {
    DeadVecResizeCloneMapItem::new(raw).dead_method()
}
