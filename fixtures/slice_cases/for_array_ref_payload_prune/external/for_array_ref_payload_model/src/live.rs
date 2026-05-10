pub fn selected_for_array_ref_payload(raw: &str) -> String {
    let items = [ForArrayRefPayloadPayload::new(raw), ForArrayRefPayloadPayload::new("tail")];
    let mut rendered = Vec::new();
    for payload in &items {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForArrayRefPayloadPayload {
    value: String,
}

impl ForArrayRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_array_ref_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_array_ref_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-array-ref-payload:{}", self.value)
    }
}

pub fn dead_live_for_array_ref_payload(raw: &str) -> String {
    ForArrayRefPayloadPayload::new(raw).unused_label()
}
