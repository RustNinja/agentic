pub struct DeadCellIntoInnerMapItem;

pub struct DeadCellIntoInnerMapPayload {
    value: String,
}

impl DeadCellIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cell-into-inner-map:{}", self.value)
    }
}

pub fn dead_cell_into_inner_map(raw: &str) -> String {
    DeadCellIntoInnerMapPayload::new(raw).dead_method()
}
