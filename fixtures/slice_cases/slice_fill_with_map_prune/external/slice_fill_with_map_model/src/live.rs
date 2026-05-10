pub struct SliceFillWithMapPayload {
    value: String,
}

impl SliceFillWithMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-fill-with-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-fill-with-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-fill-with-map:{}", self.value)
    }
}

pub fn selected_slice_fill_with_map(raw: &str) -> String {
    let mut values = [SliceFillWithMapPayload::new("dead")];
    values
        .as_mut_slice()
        .fill_with(|| SliceFillWithMapPayload::new(raw));
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_slice_fill_with_map(raw: &str) -> String {
    SliceFillWithMapPayload::new(raw).unused_label()
}
