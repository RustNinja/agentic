#[derive(Clone)]
pub struct SliceRotateRightIterPayload {
    value: String,
}

impl SliceRotateRightIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-rotate-right-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-rotate-right-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rotate-right-iter:{}", self.value)
    }
}

pub fn selected_slice_rotate_right_iter(raw: &str) -> String {
    let mut payloads = [
        SliceRotateRightIterPayload::new(raw),
        SliceRotateRightIterPayload::new("middle"),
        SliceRotateRightIterPayload::new("tail"),
    ];
    payloads.rotate_right(1);
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-rotate-right-iter:missing".to_string())
}

pub fn dead_live_slice_rotate_right_iter(raw: &str) -> String {
    SliceRotateRightIterPayload::new(raw).dead_method()
}
