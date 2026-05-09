pub struct DeadSliceRsplitnItem {
    value: String,
}

impl DeadSliceRsplitnItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rsplitn:{}", self.value)
    }
}

pub fn dead_slice_rsplitn(raw: &str) -> String {
    DeadSliceRsplitnItem::new(raw).dead_method()
}
