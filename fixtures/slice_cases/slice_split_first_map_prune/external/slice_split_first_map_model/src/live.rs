#[derive(Clone)]
pub struct SliceSplitFirstMapPayload {
    value: String,
}

impl SliceSplitFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-split-first-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-first-map:{}", self.value)
    }
}

pub fn selected_slice_split_first_map(raw: &str) -> String {
    let payloads = [
        SliceSplitFirstMapPayload::new(raw),
        SliceSplitFirstMapPayload::new("tail"),
    ];
    payloads
        .split_first()
        .map(|(payload, _tail)| payload.render_label())
        .unwrap_or_else(|| "slice-split-first-map:missing".to_string())
}

pub fn dead_live_slice_split_first_map(raw: &str) -> String {
    SliceSplitFirstMapPayload::new(raw).dead_method()
}
