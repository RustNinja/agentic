pub fn selected_for_vec_mut_payload(raw: &str) -> String {
    let mut items = for_vec_mut_payload_items(raw);
    let mut rendered = Vec::new();
    for payload in &mut items {
        rendered.push(payload.bump_and_render());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForVecMutPayloadPayload {
    value: String,
}

impl ForVecMutPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_vec_mut_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_vec_mut_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-vec-mut-payload:{}", self.value)
    }
}

fn for_vec_mut_payload_items(raw: &str) -> Vec<ForVecMutPayloadPayload> {
    vec![ForVecMutPayloadPayload::new(raw), ForVecMutPayloadPayload::new("tail")]
}

pub fn dead_live_for_vec_mut_payload(raw: &str) -> String {
    ForVecMutPayloadPayload::new(raw).unused_label()
}
