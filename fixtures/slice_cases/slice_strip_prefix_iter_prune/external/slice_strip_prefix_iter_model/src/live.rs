#[derive(Clone, PartialEq, Eq)]
pub struct SliceStripPrefixIterPayload {
    value: String,
}

impl SliceStripPrefixIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-strip-prefix-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-strip-prefix-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-strip-prefix-iter:{}", self.value)
    }
}

pub fn selected_slice_strip_prefix_iter(raw: &str) -> String {
    let prefix = [SliceStripPrefixIterPayload::new("head")];
    let payloads = [
        SliceStripPrefixIterPayload::new("head"),
        SliceStripPrefixIterPayload::new(raw),
        SliceStripPrefixIterPayload::new("tail"),
    ];
    payloads
        .strip_prefix(&prefix)
        .unwrap_or(&payloads)
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-strip-prefix-iter:missing".to_string())
}

pub fn dead_live_slice_strip_prefix_iter(raw: &str) -> String {
    SliceStripPrefixIterPayload::new(raw).dead_method()
}
