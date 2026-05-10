pub fn selected_slice_last_chunk_map(raw: &str) -> String {
    let payloads = slice_last_chunk_map_items(raw);
    payloads
        .last_chunk::<2>()
        .map(|[head, _tail]| head.render_label())
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceLastChunkMapPayload {
    value: String,
}

impl SliceLastChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_last_chunk_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_last_chunk_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-last-chunk-map:{}", self.value)
    }
}

fn slice_last_chunk_map_items(raw: &str) -> Vec<SliceLastChunkMapPayload> {
    vec![
        SliceLastChunkMapPayload::new(raw),
        SliceLastChunkMapPayload::new("middle"),
        SliceLastChunkMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_last_chunk_map(raw: &str) -> String {
    SliceLastChunkMapPayload::new(raw).unused_label()
}
