pub struct SliceSplitnMutFilterMapPayload {
    value: String,
}

impl SliceSplitnMutFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-splitn-mut-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-splitn-mut-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-splitn-mut-filter-map:{}", self.value)
    }
}

pub fn selected_slice_splitn_mut_filter_map(raw: &str) -> String {
    let mut values = [
        SliceSplitnMutFilterMapPayload::new(raw),
        SliceSplitnMutFilterMapPayload::new("skip"),
    ];
    values
        .splitn_mut(2, |payload| payload.value == "skip")
        .filter_map(|chunk| chunk.first_mut())
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_slice_splitn_mut_filter_map(raw: &str) -> String {
    SliceSplitnMutFilterMapPayload::new(raw).unused_label()
}
