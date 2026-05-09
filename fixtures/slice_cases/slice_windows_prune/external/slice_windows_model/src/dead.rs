pub struct DeadSliceWindowsItem {
    value: String,
}

impl DeadSliceWindowsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-windows:{}", self.value)
    }
}

pub fn dead_slice_windows(raw: &str) -> String {
    DeadSliceWindowsItem::new(raw).dead_method()
}
