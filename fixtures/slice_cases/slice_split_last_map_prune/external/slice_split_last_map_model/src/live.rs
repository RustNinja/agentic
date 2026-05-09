#[derive(Clone)]
pub struct SliceSplitLastMapPayload {
    value: String,
}

impl SliceSplitLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-split-last-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-last-map:{}", self.value)
    }
}

pub fn selected_slice_split_last_map(raw: &str) -> String {
    let payloads = [
        SliceSplitLastMapPayload::new("head"),
        SliceSplitLastMapPayload::new(raw),
    ];
    payloads
        .split_last()
        .map(|(payload, _tail)| payload.render_label())
        .unwrap_or_else(|| "slice-split-last-map:missing".to_string())
}

pub fn dead_live_slice_split_last_map(raw: &str) -> String {
    SliceSplitLastMapPayload::new(raw).dead_method()
}
