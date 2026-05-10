pub struct SliceChunksForPatternPayload {
    value: String,
}

impl SliceChunksForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-chunks-for-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-chunks-for-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-chunks-for-pattern:{}", self.value)
    }
}

fn slice_chunks_for_pattern_items(raw: &str) -> Vec<SliceChunksForPatternPayload> {
    vec![SliceChunksForPatternPayload::new(raw), SliceChunksForPatternPayload::new("tail")]
}

pub fn selected_slice_chunks_for_pattern(raw: &str) -> String {
    let items = slice_chunks_for_pattern_items(raw);
    for chunk in items.chunks(2) {
        let [payload, ..] = chunk else {
            continue;
        };
        return payload.render_label();
    }
    "slice-chunks-for-pattern:missing".to_string()
}

pub fn dead_live_slice_chunks_for_pattern(raw: &str) -> String {
    SliceChunksForPatternPayload::new(raw).unused_label()
}
