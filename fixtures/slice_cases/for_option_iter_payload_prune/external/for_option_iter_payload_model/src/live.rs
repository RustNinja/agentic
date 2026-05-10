pub fn selected_for_option_iter_payload(raw: &str) -> String {
    let maybe = Some(ForOptionIterPayloadPayload::new(raw));
    let mut rendered = Vec::new();
    for payload in maybe.iter() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForOptionIterPayloadPayload {
    value: String,
}

impl ForOptionIterPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_option_iter_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_option_iter_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-option-iter-payload:{}", self.value)
    }
}

pub fn dead_live_for_option_iter_payload(raw: &str) -> String {
    ForOptionIterPayloadPayload::new(raw).unused_label()
}
