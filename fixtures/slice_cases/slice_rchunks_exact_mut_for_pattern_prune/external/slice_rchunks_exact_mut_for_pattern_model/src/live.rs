pub struct SliceRchunksExactMutForPatternPayload {
    value: String,
}

impl SliceRchunksExactMutForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rchunks-exact-mut-for-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-rchunks-exact-mut-for-pattern:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-rchunks-exact-mut-for-pattern:{}", self.value)
    }
}

fn slice_rchunks_exact_mut_for_pattern_items(raw: &str) -> Vec<SliceRchunksExactMutForPatternPayload> {
    vec![SliceRchunksExactMutForPatternPayload::new(raw), SliceRchunksExactMutForPatternPayload::new("tail")]
}

pub fn selected_slice_rchunks_exact_mut_for_pattern(raw: &str) -> String {
    let mut items = slice_rchunks_exact_mut_for_pattern_items(raw);
    for chunk in items.rchunks_exact_mut(2) {
        let [payload, _tail] = chunk else {
            continue;
        };
        return payload.bump_and_render();
    }
    "slice-rchunks-exact-mut-for-pattern:missing".to_string()
}

pub fn dead_live_slice_rchunks_exact_mut_for_pattern(raw: &str) -> String {
    SliceRchunksExactMutForPatternPayload::new(raw).unused_label()
}
