pub struct SliceMutIfLetPatternPayload {
    value: String,
}

impl SliceMutIfLetPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-mut-if-let-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-mut-if-let-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-mut-if-let-pattern:{}", self.value)
    }
}

fn slice_mut_if_let_pattern_items(raw: &str) -> Vec<SliceMutIfLetPatternPayload> {
    vec![SliceMutIfLetPatternPayload::new(raw), SliceMutIfLetPatternPayload::new("tail")]
}

pub fn selected_slice_mut_if_let_pattern(raw: &str) -> String {
    let mut items = slice_mut_if_let_pattern_items(raw);
    if let [payload, ..] = items.as_mut_slice() {
        payload.bump_and_render()
    } else {
        "slice-mut-if-let-pattern:missing".to_string()
    }
}

pub fn dead_live_slice_mut_if_let_pattern(raw: &str) -> String {
    SliceMutIfLetPatternPayload::new(raw).unused_label()
}
