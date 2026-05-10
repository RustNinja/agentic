pub struct DeadSliceFillWithMapItem {
    value: String,
}

impl DeadSliceFillWithMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-fill-with-map:{}", self.value)
    }
}

pub fn dead_slice_fill_with_map(raw: &str) -> String {
    DeadSliceFillWithMapItem::new(raw).dead_method()
}
