pub struct DeadSliceSplitnItem {
    value: String,
}

impl DeadSliceSplitnItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-splitn:{}", self.value)
    }
}

pub fn dead_slice_splitn(raw: &str) -> String {
    DeadSliceSplitnItem::new(raw).dead_method()
}
