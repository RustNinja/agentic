pub struct DeadPinBoxAsMutMapItem {
    value: String,
}

impl DeadPinBoxAsMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-pin-box-as-mut-map:{}", self.value)
    }
}

pub fn dead_pin_box_as_mut_map(raw: &str) -> String {
    DeadPinBoxAsMutMapItem::new(raw).render()
}
