pub struct DeadPinBoxAsRefMapItem {
    value: String,
}

impl DeadPinBoxAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-pin-box-as-ref-map:{}", self.value)
    }
}

pub fn dead_pin_box_as_ref_map(raw: &str) -> String {
    DeadPinBoxAsRefMapItem::new(raw).render()
}
