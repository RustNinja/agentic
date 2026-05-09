pub struct SliceChunksMutFilterMapPayload {
    value: String,
}

impl SliceChunksMutFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-chunks-mut-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-chunks-mut-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-chunks-mut-filter-map:{}", self.value)
    }
}

pub fn selected_slice_chunks_mut_filter_map(raw: &str) -> String {
    let mut values = [
        SliceChunksMutFilterMapPayload::new(raw),
        SliceChunksMutFilterMapPayload::new("tail"),
    ];
    values
        .chunks_mut(1)
        .filter_map(|chunk| chunk.first_mut())
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_slice_chunks_mut_filter_map(raw: &str) -> String {
    SliceChunksMutFilterMapPayload::new(raw).unused_label()
}
