pub struct DeadSliceStripPrefixIterItem {
    value: String,
}

impl DeadSliceStripPrefixIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-strip-prefix-iter:{}", self.value)
    }
}

pub fn dead_slice_strip_prefix_iter(raw: &str) -> String {
    DeadSliceStripPrefixIterItem::new(raw).render()
}
