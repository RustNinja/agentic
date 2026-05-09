pub struct DeadRcAsRefMapItem {
    value: String,
}

impl DeadRcAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-rc-as-ref-map:{}", self.value)
    }
}

pub fn dead_rc_as_ref_map(raw: &str) -> String {
    DeadRcAsRefMapItem::new(raw).render()
}
