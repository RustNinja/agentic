pub struct DeadSliceWindowsFilterMapItem {
    value: String,
}

impl DeadSliceWindowsFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-windows-filter-map:{}", self.value)
    }
}

pub fn dead_slice_windows_filter_map(raw: &str) -> String {
    DeadSliceWindowsFilterMapItem::new(raw).dead_method()
}
