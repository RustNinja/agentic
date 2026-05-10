pub struct DeadCellTakeMapItem;

pub struct DeadCellTakeMapPayload {
    value: String,
}

impl DeadCellTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cell-take-map:{}", self.value)
    }
}

pub fn dead_cell_take_map(raw: &str) -> String {
    DeadCellTakeMapPayload::new(raw).dead_method()
}
