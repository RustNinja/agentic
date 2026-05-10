pub struct DeadCellSetGetMapItem {
    value: String,
}

impl DeadCellSetGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cell-set-get-map:{}", self.value)
    }
}

pub fn dead_cell_set_get_map(raw: &str) -> String {
    DeadCellSetGetMapItem::new(raw).dead_method()
}
