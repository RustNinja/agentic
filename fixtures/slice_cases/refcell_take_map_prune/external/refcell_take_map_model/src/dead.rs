pub struct DeadRefcellTakeMapItem;

pub struct DeadRefcellTakeMapPayload {
    value: String,
}

impl DeadRefcellTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-take-map:{}", self.value)
    }
}

pub fn dead_refcell_take_map(raw: &str) -> String {
    DeadRefcellTakeMapPayload::new(raw).dead_method()
}
