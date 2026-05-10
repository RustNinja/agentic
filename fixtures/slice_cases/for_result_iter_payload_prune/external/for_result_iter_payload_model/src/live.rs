pub fn selected_for_result_iter_payload(raw: &str) -> String {
    let result: Result<ForResultIterPayloadPayload, ()> = Ok(ForResultIterPayloadPayload::new(raw));
    let mut rendered = Vec::new();
    for payload in result.iter() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForResultIterPayloadPayload {
    value: String,
}

impl ForResultIterPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_result_iter_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_result_iter_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-result-iter-payload:{}", self.value)
    }
}

pub fn dead_live_for_result_iter_payload(raw: &str) -> String {
    ForResultIterPayloadPayload::new(raw).unused_label()
}
