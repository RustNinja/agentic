pub struct SliceIfLetPatternPayload {
    value: String,
}

impl SliceIfLetPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-if-let-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-if-let-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-if-let-pattern:{}", self.value)
    }
}

fn slice_if_let_pattern_items(raw: &str) -> Vec<SliceIfLetPatternPayload> {
    vec![SliceIfLetPatternPayload::new(raw), SliceIfLetPatternPayload::new("tail")]
}

pub fn selected_slice_if_let_pattern(raw: &str) -> String {
    let items = slice_if_let_pattern_items(raw);
    if let [payload, ..] = items.as_slice() {
        payload.render_label()
    } else {
        "slice-if-let-pattern:missing".to_string()
    }
}

pub fn dead_live_slice_if_let_pattern(raw: &str) -> String {
    SliceIfLetPatternPayload::new(raw).unused_label()
}
