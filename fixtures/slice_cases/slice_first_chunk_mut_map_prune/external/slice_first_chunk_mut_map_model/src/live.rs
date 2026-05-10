pub fn selected_slice_first_chunk_mut_map(raw: &str) -> String {
    let mut payloads = slice_first_chunk_mut_map_items(raw);
    payloads
        .first_chunk_mut::<2>()
        .map(|[head, _tail]| head.bump_and_render())
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceFirstChunkMutMapPayload {
    value: String,
}

impl SliceFirstChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_first_chunk_mut_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_first_chunk_mut_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-first-chunk-mut-map:{}", self.value)
    }
}

fn slice_first_chunk_mut_map_items(raw: &str) -> Vec<SliceFirstChunkMutMapPayload> {
    vec![
        SliceFirstChunkMutMapPayload::new(raw),
        SliceFirstChunkMutMapPayload::new("middle"),
        SliceFirstChunkMutMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_first_chunk_mut_map(raw: &str) -> String {
    SliceFirstChunkMutMapPayload::new(raw).unused_label()
}
