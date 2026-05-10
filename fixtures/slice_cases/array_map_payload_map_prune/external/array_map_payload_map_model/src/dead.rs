pub struct DeadArrayMapPayloadMapItem {
    value: String,
}

impl DeadArrayMapPayloadMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-array-map-payload-map:{}", self.value)
    }
}

pub fn dead_array_map_payload_map(raw: &str) -> String {
    DeadArrayMapPayloadMapItem::new(raw).dead_method()
}
