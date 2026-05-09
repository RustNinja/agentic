pub struct DeadSliceStripSuffixIterItem {
    value: String,
}

impl DeadSliceStripSuffixIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-strip-suffix-iter:{}", self.value)
    }
}

pub fn dead_slice_strip_suffix_iter(raw: &str) -> String {
    DeadSliceStripSuffixIterItem::new(raw).render()
}
