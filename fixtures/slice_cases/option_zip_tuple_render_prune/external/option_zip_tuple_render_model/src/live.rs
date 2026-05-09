pub struct OptionZipTupleRenderPayload {
    value: String,
}

impl OptionZipTupleRenderPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-zip-tuple-render:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-option-zip-tuple-render:{}", self.value)
    }
}

pub fn selected_option_zip_tuple_render(raw: &str) -> String {
    let left = Some(OptionZipTupleRenderPayload::new(raw));
    let right = Some(OptionZipTupleRenderPayload::new("right"));
    left.zip(right)
        .map(|(left, right)| format!("{}|{}", left.render_label(), right.render_label()))
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_option_zip_tuple_render(raw: &str) -> String {
    OptionZipTupleRenderPayload::new(raw).unused_label()
}
