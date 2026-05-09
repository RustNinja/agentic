pub struct DeadMemReplaceMapItem {
    value: String,
}

impl DeadMemReplaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-mem-replace-map:{}", self.value)
    }
}

pub fn dead_mem_replace_map(raw: &str) -> String {
    DeadMemReplaceMapItem::new(raw).render()
}
