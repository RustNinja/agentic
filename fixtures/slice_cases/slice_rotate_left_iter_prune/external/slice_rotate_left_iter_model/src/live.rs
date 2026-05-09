#[derive(Clone)]
pub struct SliceRotateLeftIterPayload {
    value: String,
}

impl SliceRotateLeftIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rotate-left-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-rotate-left-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rotate-left-iter:{}", self.value)
    }
}

pub fn selected_slice_rotate_left_iter(raw: &str) -> String {
    let mut payloads = [
        SliceRotateLeftIterPayload::new(raw),
        SliceRotateLeftIterPayload::new("middle"),
        SliceRotateLeftIterPayload::new("tail"),
    ];
    payloads.rotate_left(1);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-rotate-left-iter:missing".to_string())
}

pub fn dead_live_slice_rotate_left_iter(raw: &str) -> String {
    SliceRotateLeftIterPayload::new(raw).dead_method()
}
