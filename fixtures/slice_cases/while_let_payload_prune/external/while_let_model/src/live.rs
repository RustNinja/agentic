pub struct WhileLetPayload {
    value: String,
}

impl WhileLetPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("while-let:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-while-let:{}", self.value)
    }
}

fn while_let_payloads(raw: &str) -> Vec<WhileLetPayload> {
    raw.split(',').map(WhileLetPayload::new).collect()
}

pub fn selected_while_let(raw: &str) -> String {
    let mut items = while_let_payloads(raw).into_iter();
    let mut rendered = Vec::new();
    while let Some(payload) = items.next() {
        rendered.push(payload.render_label());
    }
    rendered.join("|")
}

pub fn dead_live_while_let(raw: &str) -> String {
    WhileLetPayload::new(raw).dead_method()
}
