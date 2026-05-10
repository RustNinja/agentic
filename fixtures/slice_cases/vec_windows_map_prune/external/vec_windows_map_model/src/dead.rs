pub struct DeadVecWindowsMapItem {
    value: String,
}

impl DeadVecWindowsMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-windows-map:{}", self.value)
    }
}

pub fn dead_vec_windows_map(raw: &str) -> String {
    DeadVecWindowsMapItem::new(raw).dead_method()
}
