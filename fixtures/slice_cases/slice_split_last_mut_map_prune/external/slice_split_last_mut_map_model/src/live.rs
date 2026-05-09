#[derive(Clone)]
pub struct SliceSplitLastMutMapPayload {
    value: String,
}

impl SliceSplitLastMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-last-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-split-last-mut-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-last-mut-map:{}", self.value)
    }
}

pub fn selected_slice_split_last_mut_map(raw: &str) -> String {
    let mut payloads = [
        SliceSplitLastMutMapPayload::new("head"),
        SliceSplitLastMutMapPayload::new(raw),
    ];
    payloads
        .split_last_mut()
        .map(|(payload, _tail)| payload.bump_and_render())
        .unwrap_or_else(|| "slice-split-last-mut-map:missing".to_string())
}

pub fn dead_live_slice_split_last_mut_map(raw: &str) -> String {
    SliceSplitLastMutMapPayload::new(raw).dead_method()
}
