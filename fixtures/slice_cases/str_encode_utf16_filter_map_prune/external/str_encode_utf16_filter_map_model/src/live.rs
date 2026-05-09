pub struct StrEncodeUtf16FilterMapPayload {
    value: String,
}

impl StrEncodeUtf16FilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-encode-utf16-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-encode-utf16-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-encode-utf16-filter-map:{}", self.value)
    }
}

pub fn selected_str_encode_utf16_filter_map(raw: &str) -> String {
    raw.encode_utf16()
        .filter_map(|unit| {
            (unit > 0).then(|| {
                let buffer = unit.to_string();
                StrEncodeUtf16FilterMapPayload::new(&buffer).render_label()
            })
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_encode_utf16_filter_map(raw: &str) -> String {
    StrEncodeUtf16FilterMapPayload::new(raw).unused_label()
}
