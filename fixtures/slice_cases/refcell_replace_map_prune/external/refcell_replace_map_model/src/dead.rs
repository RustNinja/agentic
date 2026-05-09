pub struct DeadRefCellReplaceMapItem {
    value: String,
}

impl DeadRefCellReplaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-refcell-replace-map:{}", self.value)
    }
}

pub fn dead_refcell_replace_map(raw: &str) -> String {
    DeadRefCellReplaceMapItem::new(raw).render()
}
