pub struct StrStripSuffixMapPayload {
    value: String,
}

impl StrStripSuffixMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-strip-suffix-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-strip-suffix-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-strip-suffix-map:{}", self.value)
    }
}

pub fn selected_str_strip_suffix_map(raw: &str) -> String {
    raw.strip_suffix(".json")
        .map(|part| StrStripSuffixMapPayload::new(part).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_strip_suffix_map(raw: &str) -> String {
    StrStripSuffixMapPayload::new(raw).unused_label()
}
