#[derive(Clone, PartialEq, Eq)]
pub struct SliceStripSuffixIterPayload {
    value: String,
}

impl SliceStripSuffixIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-strip-suffix-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-strip-suffix-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-strip-suffix-iter:{}", self.value)
    }
}

pub fn selected_slice_strip_suffix_iter(raw: &str) -> String {
    let suffix = [SliceStripSuffixIterPayload::new("tail")];
    let payloads = [
        SliceStripSuffixIterPayload::new(raw),
        SliceStripSuffixIterPayload::new("middle"),
        SliceStripSuffixIterPayload::new("tail"),
    ];
    payloads
        .strip_suffix(&suffix)
        .unwrap_or(&payloads)
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-strip-suffix-iter:missing".to_string())
}

pub fn dead_live_slice_strip_suffix_iter(raw: &str) -> String {
    SliceStripSuffixIterPayload::new(raw).dead_method()
}
