pub struct DeadOptionAsPinMutMapItem;

pub struct DeadOptionAsPinMutMapPayload {
    value: String,
}

impl DeadOptionAsPinMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-as-pin-mut-map:{}", self.value)
    }
}

pub fn dead_option_as_pin_mut_map(raw: &str) -> String {
    DeadOptionAsPinMutMapPayload::new(raw).dead_method()
}
