pub struct ArrayMapPayloadMapPayload {
    value: String,
}

impl ArrayMapPayloadMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("array-map-payload-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("array-map-payload-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-array-map-payload-map:{}", self.value)
    }
}

pub fn selected_array_map_payload_map(raw: &str) -> String {
    [
        ArrayMapPayloadMapPayload::new(raw),
        ArrayMapPayloadMapPayload::new("tail"),
    ]
    .map(|payload| payload.render_label())
    .join("|")
}

pub fn dead_live_array_map_payload_map(raw: &str) -> String {
    ArrayMapPayloadMapPayload::new(raw).unused_label()
}
