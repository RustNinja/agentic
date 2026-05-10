#[derive(Clone)]
pub struct SliceFillMapPayload {
    value: String,
}

impl SliceFillMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-fill-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-fill-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-fill-map:{}", self.value)
    }
}

pub fn selected_slice_fill_map(raw: &str) -> String {
    let mut values = [SliceFillMapPayload::new("dead")];
    values.as_mut_slice().fill(SliceFillMapPayload::new(raw));
    values
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_slice_fill_map(raw: &str) -> String {
    SliceFillMapPayload::new(raw).unused_label()
}
