use std::collections::BTreeSet;

pub fn selected_for_btreeset_ref_payload(raw: &str) -> String {
    let items: BTreeSet<_> = for_btreeset_ref_payload_items(raw).into_iter().collect();
    let mut rendered = Vec::new();
    for payload in &items {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForBtreesetRefPayloadPayload {
    value: String,
}

impl ForBtreesetRefPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_btreeset_ref_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_btreeset_ref_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-btreeset-ref-payload:{}", self.value)
    }
}

fn for_btreeset_ref_payload_items(raw: &str) -> Vec<ForBtreesetRefPayloadPayload> {
    vec![ForBtreesetRefPayloadPayload::new(raw), ForBtreesetRefPayloadPayload::new("tail")]
}

pub fn dead_live_for_btreeset_ref_payload(raw: &str) -> String {
    ForBtreesetRefPayloadPayload::new(raw).unused_label()
}
