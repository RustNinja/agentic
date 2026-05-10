pub struct SliceLetElsePatternPayload {
    value: String,
}

impl SliceLetElsePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-let-else-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-let-else-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-let-else-pattern:{}", self.value)
    }
}

fn slice_let_else_pattern_items(raw: &str) -> Vec<SliceLetElsePatternPayload> {
    vec![SliceLetElsePatternPayload::new(raw), SliceLetElsePatternPayload::new("tail")]
}

pub fn selected_slice_let_else_pattern(raw: &str) -> String {
    let items = slice_let_else_pattern_items(raw);
    let [payload, ..] = items.as_slice() else {
        return "slice-let-else-pattern:missing".to_string();
    };
    payload.render_label()
}

pub fn dead_live_slice_let_else_pattern(raw: &str) -> String {
    SliceLetElsePatternPayload::new(raw).unused_label()
}
