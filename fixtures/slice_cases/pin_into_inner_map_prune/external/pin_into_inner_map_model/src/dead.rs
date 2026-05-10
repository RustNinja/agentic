pub struct DeadPinIntoInnerMapItem {
    value: String,
}

impl DeadPinIntoInnerMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-pin-into-inner-map:{}", self.value)
    }
}

pub fn dead_pin_into_inner_map(raw: &str) -> String {
    DeadPinIntoInnerMapItem::new(raw).dead_method()
}
