pub struct ForLoopPayload {
    value: String,
}

impl ForLoopPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("for-loop:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-loop:{}", self.value)
    }
}

fn for_loop_payloads(raw: &str) -> Vec<ForLoopPayload> {
    raw.split(',').map(ForLoopPayload::new).collect()
}

pub fn selected_for_loop(raw: &str) -> String {
    let mut rendered = Vec::new();
    for payload in for_loop_payloads(raw) {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

pub fn dead_live_for_loop(raw: &str) -> String {
    ForLoopPayload::new(raw).dead_method()
}
