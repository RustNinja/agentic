pub struct DeadSliceFillMapItem {
    value: String,
}

impl DeadSliceFillMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-fill-map:{}", self.value)
    }
}

pub fn dead_slice_fill_map(raw: &str) -> String {
    DeadSliceFillMapItem::new(raw).dead_method()
}
