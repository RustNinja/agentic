pub struct DeadBoxPinAsRefMapItem {
    value: String,
}

impl DeadBoxPinAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-box-pin-as-ref-map:{}", self.value)
    }
}

pub fn dead_box_pin_as_ref_map(raw: &str) -> String {
    DeadBoxPinAsRefMapItem::new(raw).render()
}
