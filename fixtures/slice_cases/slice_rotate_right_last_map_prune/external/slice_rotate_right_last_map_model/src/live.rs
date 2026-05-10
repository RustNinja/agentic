pub struct SliceRotateRightLastMapPayload {
    value: String,
}

impl SliceRotateRightLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rotate-right-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-rotate-right-last-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-rotate-right-last-map:{}", self.value)
    }
}

pub fn selected_slice_rotate_right_last_map(raw: &str) -> String {
    let mut values = [
        SliceRotateRightLastMapPayload::new(raw),
        SliceRotateRightLastMapPayload::new("tail"),
    ];
    values.rotate_right(1);
    values
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_slice_rotate_right_last_map(raw: &str) -> String {
    SliceRotateRightLastMapPayload::new(raw).unused_label()
}
