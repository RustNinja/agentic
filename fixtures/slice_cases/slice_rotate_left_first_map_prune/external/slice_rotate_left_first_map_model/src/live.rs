pub struct SliceRotateLeftFirstMapPayload {
    value: String,
}

impl SliceRotateLeftFirstMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rotate-left-first-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice-rotate-left-first-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-rotate-left-first-map:{}", self.value)
    }
}

pub fn selected_slice_rotate_left_first_map(raw: &str) -> String {
    let mut values = [
        SliceRotateLeftFirstMapPayload::new("head"),
        SliceRotateLeftFirstMapPayload::new(raw),
    ];
    values.rotate_left(1);
    values
        .first()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_slice_rotate_left_first_map(raw: &str) -> String {
    SliceRotateLeftFirstMapPayload::new(raw).unused_label()
}
