pub struct SliceRchunksMutFilterMapPayload {
    value: String,
}

impl SliceRchunksMutFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rchunks-mut-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-rchunks-mut-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-rchunks-mut-filter-map:{}", self.value)
    }
}

pub fn selected_slice_rchunks_mut_filter_map(raw: &str) -> String {
    let mut values = [
        SliceRchunksMutFilterMapPayload::new("head"),
        SliceRchunksMutFilterMapPayload::new(raw),
    ];
    values
        .rchunks_mut(1)
        .filter_map(|chunk| chunk.first_mut())
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_slice_rchunks_mut_filter_map(raw: &str) -> String {
    SliceRchunksMutFilterMapPayload::new(raw).unused_label()
}
