pub fn selected_for_result_iter_mut_payload(raw: &str) -> String {
    let mut result: Result<ForResultIterMutPayloadPayload, ()> = Ok(ForResultIterMutPayloadPayload::new(raw));
    let mut rendered = Vec::new();
    for payload in result.iter_mut() {
        rendered.push(payload.bump_and_render());
    }
    rendered.join("|")
}

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ForResultIterMutPayloadPayload {
    value: String,
}

impl ForResultIterMutPayloadPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for_result_iter_mut_payload:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("for_result_iter_mut_payload:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-for-result-iter-mut-payload:{}", self.value)
    }
}

pub fn dead_live_for_result_iter_mut_payload(raw: &str) -> String {
    ForResultIterMutPayloadPayload::new(raw).unused_label()
}
