pub struct SliceChunksMutForPatternPayload {
    value: String,
}

impl SliceChunksMutForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-chunks-mut-for-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-chunks-mut-for-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-chunks-mut-for-pattern:{}", self.value)
    }
}

fn slice_chunks_mut_for_pattern_items(raw: &str) -> Vec<SliceChunksMutForPatternPayload> {
    vec![SliceChunksMutForPatternPayload::new(raw), SliceChunksMutForPatternPayload::new("tail")]
}

pub fn selected_slice_chunks_mut_for_pattern(raw: &str) -> String {
    let mut items = slice_chunks_mut_for_pattern_items(raw);
    for chunk in items.chunks_mut(2) {
        let [payload, ..] = chunk else {
            continue;
        };
        return payload.bump_and_render();
    }
    "slice-chunks-mut-for-pattern:missing".to_string()
}

pub fn dead_live_slice_chunks_mut_for_pattern(raw: &str) -> String {
    SliceChunksMutForPatternPayload::new(raw).unused_label()
}
