pub struct SliceRchunksForPatternPayload {
    value: String,
}

impl SliceRchunksForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rchunks-for-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-rchunks-for-pattern:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-rchunks-for-pattern:{}", self.value)
    }
}

fn slice_rchunks_for_pattern_items(raw: &str) -> Vec<SliceRchunksForPatternPayload> {
    vec![SliceRchunksForPatternPayload::new(raw), SliceRchunksForPatternPayload::new("tail")]
}

pub fn selected_slice_rchunks_for_pattern(raw: &str) -> String {
    let items = slice_rchunks_for_pattern_items(raw);
    for chunk in items.rchunks(2) {
        let [payload, ..] = chunk else {
            continue;
        };
        return payload.render_label();
    }
    "slice-rchunks-for-pattern:missing".to_string()
}

pub fn dead_live_slice_rchunks_for_pattern(raw: &str) -> String {
    SliceRchunksForPatternPayload::new(raw).unused_label()
}
