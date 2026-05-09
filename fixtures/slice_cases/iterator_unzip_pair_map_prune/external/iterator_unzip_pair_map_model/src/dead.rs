pub struct DeadIteratorUnzipPairMapItem {
    value: String,
}

impl DeadIteratorUnzipPairMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-unzip-pair-map:{}", self.value)
    }
}

pub fn dead_iterator_unzip_pair_map(raw: &str) -> String {
    DeadIteratorUnzipPairMapItem::new(raw).render()
}
