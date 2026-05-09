#[derive(Clone)]
pub struct SliceReverseIterPayload {
    value: String,
}

impl SliceReverseIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-reverse-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-reverse-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-reverse-iter:{}", self.value)
    }
}

pub fn selected_slice_reverse_iter(raw: &str) -> String {
    let mut payloads = [
        SliceReverseIterPayload::new(raw),
        SliceReverseIterPayload::new("tail"),
    ];
    payloads.reverse();
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-reverse-iter:missing".to_string())
}

pub fn dead_live_slice_reverse_iter(raw: &str) -> String {
    SliceReverseIterPayload::new(raw).dead_method()
}
