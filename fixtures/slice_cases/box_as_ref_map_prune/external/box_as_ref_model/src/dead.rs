pub struct DeadBoxAsRefMapItem {
    value: String,
}

impl DeadBoxAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-box-as-ref-map:{}", self.value)
    }
}

pub fn dead_box_as_ref_map(raw: &str) -> String {
    DeadBoxAsRefMapItem::new(raw).render()
}
