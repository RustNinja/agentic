pub struct SliceChunkByMutFlattenMapPayload {
    value: String,
}

impl SliceChunkByMutFlattenMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-chunk-by-mut-flatten-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-chunk-by-mut-flatten-map:{}", self.value)
    }
    pub fn group_key(&self) -> &str {
        &self.value
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-chunk-by-mut-flatten-map:{}", self.value)
    }
}

pub fn selected_slice_chunk_by_mut_flatten_map(raw: &str) -> String {
    let mut items = vec![SliceChunkByMutFlattenMapPayload::new(raw), SliceChunkByMutFlattenMapPayload::new(raw)];
    items
        .as_mut_slice()
        .chunk_by_mut(|left, right| left.group_key() == right.group_key())
        .flatten()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_slice_chunk_by_mut_flatten_map(raw: &str) -> String {
    SliceChunkByMutFlattenMapPayload::new(raw).unused_label()
}
