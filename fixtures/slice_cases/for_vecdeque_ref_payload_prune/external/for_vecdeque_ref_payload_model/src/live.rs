use std::collections::VecDeque;

pub fn selected_for_vecdeque_ref_payload(raw: &str) -> String {
    let items = VecDeque::from(for_vecdeque_ref_payload_items(raw));
    let mut rendered = Vec::new();
    for payload in &items {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForVecdequeRefPayloadPayload {
    value: String,
}

impl ForVecdequeRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_vecdeque_ref_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_vecdeque_ref_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-vecdeque-ref-payload:{}", self.value)
    }
}

fn for_vecdeque_ref_payload_items(raw: &str) -> Vec<ForVecdequeRefPayloadPayload> {
    vec![ForVecdequeRefPayloadPayload::new(raw), ForVecdequeRefPayloadPayload::new("tail")]
}

pub fn dead_live_for_vecdeque_ref_payload(raw: &str) -> String {
    ForVecdequeRefPayloadPayload::new(raw).unused_label()
}
