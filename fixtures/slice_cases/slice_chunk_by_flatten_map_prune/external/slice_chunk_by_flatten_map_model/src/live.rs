pub struct SliceChunkByFlattenMapPayload {
    value: String,
}

impl SliceChunkByFlattenMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-chunk-by-flatten-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-chunk-by-flatten-map:{}", self.value)
    }
    pub fn group_key(&self) -> &str {
        &self.value
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-chunk-by-flatten-map:{}", self.value)
    }
}

pub fn selected_slice_chunk_by_flatten_map(raw: &str) -> String {
    let items = vec![SliceChunkByFlattenMapPayload::new(raw), SliceChunkByFlattenMapPayload::new(raw)];
    items
        .as_slice()
        .chunk_by(|left, right| left.group_key() == right.group_key())
        .flatten()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_slice_chunk_by_flatten_map(raw: &str) -> String {
    SliceChunkByFlattenMapPayload::new(raw).unused_label()
}
