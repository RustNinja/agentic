use std::collections::HashSet;

pub fn selected_for_hashset_ref_payload(raw: &str) -> String {
    let items: HashSet<_> = for_hashset_ref_payload_items(raw).into_iter().collect();
    let mut rendered = Vec::new();
    for payload in &items {
        rendered.push(payload.render_label());
    }
    rendered.sort();
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForHashsetRefPayloadPayload {
    value: String,
}

impl ForHashsetRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_hashset_ref_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_hashset_ref_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-hashset-ref-payload:{}", self.value)
    }
}

fn for_hashset_ref_payload_items(raw: &str) -> Vec<ForHashsetRefPayloadPayload> {
    vec![ForHashsetRefPayloadPayload::new(raw), ForHashsetRefPayloadPayload::new("tail")]
}

pub fn dead_live_for_hashset_ref_payload(raw: &str) -> String {
    ForHashsetRefPayloadPayload::new(raw).unused_label()
}
