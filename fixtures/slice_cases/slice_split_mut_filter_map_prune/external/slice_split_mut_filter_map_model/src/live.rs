pub struct SliceSplitMutFilterMapPayload {
    value: String,
}

impl SliceSplitMutFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-mut-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-split-mut-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-mut-filter-map:{}", self.value)
    }
}

pub fn selected_slice_split_mut_filter_map(raw: &str) -> String {
    let mut values = [
        SliceSplitMutFilterMapPayload::new(raw),
        SliceSplitMutFilterMapPayload::new("skip"),
    ];
    values
        .split_mut(|payload| payload.value == "skip")
        .filter_map(|chunk| chunk.first_mut())
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_slice_split_mut_filter_map(raw: &str) -> String {
    SliceSplitMutFilterMapPayload::new(raw).unused_label()
}
