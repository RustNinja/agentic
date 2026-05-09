pub struct DeadManuallyDropIntoInnerMapItem {
    value: String,
}

impl DeadManuallyDropIntoInnerMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-manuallydrop-into-inner-map:{}", self.value)
    }
}

pub fn dead_manuallydrop_into_inner_map(raw: &str) -> String {
    DeadManuallyDropIntoInnerMapItem::new(raw).render()
}
