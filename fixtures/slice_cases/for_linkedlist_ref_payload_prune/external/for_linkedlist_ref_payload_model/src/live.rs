use std::collections::LinkedList;

pub fn selected_for_linkedlist_ref_payload(raw: &str) -> String {
    let items: LinkedList<_> = for_linkedlist_ref_payload_items(raw).into_iter().collect();
    let mut rendered = Vec::new();
    for payload in &items {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForLinkedlistRefPayloadPayload {
    value: String,
}

impl ForLinkedlistRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_linkedlist_ref_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_linkedlist_ref_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-linkedlist-ref-payload:{}", self.value)
    }
}

fn for_linkedlist_ref_payload_items(raw: &str) -> Vec<ForLinkedlistRefPayloadPayload> {
    vec![ForLinkedlistRefPayloadPayload::new(raw), ForLinkedlistRefPayloadPayload::new("tail")]
}

pub fn dead_live_for_linkedlist_ref_payload(raw: &str) -> String {
    ForLinkedlistRefPayloadPayload::new(raw).unused_label()
}
