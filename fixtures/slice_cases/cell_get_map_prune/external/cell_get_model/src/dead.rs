pub struct DeadCellGetMapItem {
    value: String,
}

impl DeadCellGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-cell-get-map:{}", self.value)
    }
}

pub fn dead_cell_get_map(raw: &str) -> String {
    DeadCellGetMapItem::new(raw).render()
}
