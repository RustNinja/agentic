pub struct DeadRefcellIntoInnerMapItem;

pub struct DeadRefcellIntoInnerMapPayload {
    value: String,
}

impl DeadRefcellIntoInnerMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-refcell-into-inner-map:{}", self.value)
    }
}

pub fn dead_refcell_into_inner_map(raw: &str) -> String {
    DeadRefcellIntoInnerMapPayload::new(raw).dead_method()
}
