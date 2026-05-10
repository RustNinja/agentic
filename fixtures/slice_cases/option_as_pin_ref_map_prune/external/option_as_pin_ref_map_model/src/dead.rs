pub struct DeadOptionAsPinRefMapItem;

pub struct DeadOptionAsPinRefMapPayload {
    value: String,
}

impl DeadOptionAsPinRefMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-pin-ref-map:{}", self.value)
    }
}

pub fn dead_option_as_pin_ref_map(raw: &str) -> String {
    DeadOptionAsPinRefMapPayload::new(raw).dead_method()
}
