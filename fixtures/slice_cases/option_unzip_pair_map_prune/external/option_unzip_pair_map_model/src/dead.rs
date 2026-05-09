pub struct DeadOptionUnzipPairMapItem {
    value: String,
}

impl DeadOptionUnzipPairMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-unzip-pair-map:{}", self.value)
    }
}

pub fn dead_option_unzip_pair_map(raw: &str) -> String {
    DeadOptionUnzipPairMapItem::new(raw).render()
}
