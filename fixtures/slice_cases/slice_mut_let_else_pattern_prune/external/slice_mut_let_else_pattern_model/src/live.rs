pub struct SliceMutLetElsePatternPayload {
    value: String,
}

impl SliceMutLetElsePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-mut-let-else-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-mut-let-else-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-mut-let-else-pattern:{}", self.value)
    }
}

fn slice_mut_let_else_pattern_items(raw: &str) -> Vec<SliceMutLetElsePatternPayload> {
    vec![SliceMutLetElsePatternPayload::new(raw), SliceMutLetElsePatternPayload::new("tail")]
}

pub fn selected_slice_mut_let_else_pattern(raw: &str) -> String {
    let mut items = slice_mut_let_else_pattern_items(raw);
    let [payload, ..] = items.as_mut_slice() else {
        return "slice-mut-let-else-pattern:missing".to_string();
    };
    payload.bump_and_render()
}

pub fn dead_live_slice_mut_let_else_pattern(raw: &str) -> String {
    SliceMutLetElsePatternPayload::new(raw).unused_label()
}
