#[derive(Clone)]
pub struct SliceSplitFirstMutMapPayload {
    value: String,
}

impl SliceSplitFirstMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-first-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-split-first-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-first-mut-map:{}", self.value)
    }
}

pub fn selected_slice_split_first_mut_map(raw: &str) -> String {
    let mut payloads = [
        SliceSplitFirstMutMapPayload::new(raw),
        SliceSplitFirstMutMapPayload::new("tail"),
    ];
    payloads
        .split_first_mut()
        .map(|(payload, _tail)| payload.bump_and_render())
        .unwrap_or_else(|| "slice-split-first-mut-map:missing".to_string())
}

pub fn dead_live_slice_split_first_mut_map(raw: &str) -> String {
    SliceSplitFirstMutMapPayload::new(raw).dead_method()
}
