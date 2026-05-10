pub fn selected_for_vec_ref_payload(raw: &str) -> String {
    let items = for_vec_ref_payload_items(raw);
    let mut rendered = Vec::new();
    for payload in &items {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForVecRefPayloadPayload {
    value: String,
}

impl ForVecRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_vec_ref_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_vec_ref_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-vec-ref-payload:{}", self.value)
    }
}

fn for_vec_ref_payload_items(raw: &str) -> Vec<ForVecRefPayloadPayload> {
    vec![ForVecRefPayloadPayload::new(raw), ForVecRefPayloadPayload::new("tail")]
}

pub fn dead_live_for_vec_ref_payload(raw: &str) -> String {
    ForVecRefPayloadPayload::new(raw).unused_label()
}
