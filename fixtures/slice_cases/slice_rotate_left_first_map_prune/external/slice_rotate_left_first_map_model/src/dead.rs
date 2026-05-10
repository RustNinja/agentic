pub struct DeadSliceRotateLeftFirstMapItem {
    value: String,
}

impl DeadSliceRotateLeftFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rotate-left-first-map:{}", self.value)
    }
}

pub fn dead_slice_rotate_left_first_map(raw: &str) -> String {
    DeadSliceRotateLeftFirstMapItem::new(raw).dead_method()
}
