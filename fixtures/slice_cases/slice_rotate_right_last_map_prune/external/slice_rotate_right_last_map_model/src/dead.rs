pub struct DeadSliceRotateRightLastMapItem {
    value: String,
}

impl DeadSliceRotateRightLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rotate-right-last-map:{}", self.value)
    }
}

pub fn dead_slice_rotate_right_last_map(raw: &str) -> String {
    DeadSliceRotateRightLastMapItem::new(raw).dead_method()
}
