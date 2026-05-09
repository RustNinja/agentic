pub struct DeadCellReplaceMapItem {
    value: String,
}

impl DeadCellReplaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-cell-replace-map:{}", self.value)
    }
}

pub fn dead_cell_replace_map(raw: &str) -> String {
    DeadCellReplaceMapItem::new(raw).render()
}
