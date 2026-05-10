pub fn selected_slice_first_chunk_map(raw: &str) -> String {
    let payloads = slice_first_chunk_map_items(raw);
    payloads
        .first_chunk::<2>()
        .map(|[head, _tail]| head.render_label())
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceFirstChunkMapPayload {
    value: String,
}

impl SliceFirstChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_first_chunk_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_first_chunk_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-first-chunk-map:{}", self.value)
    }
}

fn slice_first_chunk_map_items(raw: &str) -> Vec<SliceFirstChunkMapPayload> {
    vec![
        SliceFirstChunkMapPayload::new(raw),
        SliceFirstChunkMapPayload::new("middle"),
        SliceFirstChunkMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_first_chunk_map(raw: &str) -> String {
    SliceFirstChunkMapPayload::new(raw).unused_label()
}
