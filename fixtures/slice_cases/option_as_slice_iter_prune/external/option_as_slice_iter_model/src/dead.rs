pub struct DeadOptionAsSliceIterItem {
    value: String,
}

impl DeadOptionAsSliceIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-as-slice-iter:{}", self.value)
    }
}

pub fn dead_option_as_slice_iter(raw: &str) -> String {
    DeadOptionAsSliceIterItem::new(raw).render()
}
