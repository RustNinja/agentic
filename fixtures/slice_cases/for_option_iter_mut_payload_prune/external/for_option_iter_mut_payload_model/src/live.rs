pub fn selected_for_option_iter_mut_payload(raw: &str) -> String {
    let mut maybe = Some(ForOptionIterMutPayloadPayload::new(raw));
    let mut rendered = Vec::new();
    for payload in maybe.iter_mut() {
        rendered.push(payload.bump_and_render());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForOptionIterMutPayloadPayload {
    value: String,
}

impl ForOptionIterMutPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_option_iter_mut_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_option_iter_mut_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-option-iter-mut-payload:{}", self.value)
    }
}

pub fn dead_live_for_option_iter_mut_payload(raw: &str) -> String {
    ForOptionIterMutPayloadPayload::new(raw).unused_label()
}
