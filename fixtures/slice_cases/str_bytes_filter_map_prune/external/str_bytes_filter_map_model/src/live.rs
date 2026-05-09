pub struct StrBytesFilterMapPayload {
    value: String,
}

impl StrBytesFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-bytes-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-bytes-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-bytes-filter-map:{}", self.value)
    }
}

pub fn selected_str_bytes_filter_map(raw: &str) -> String {
    raw.bytes()
        .filter_map(|byte| {
            if byte.is_ascii_digit() {
                let buffer = byte.to_string();
                Some(StrBytesFilterMapPayload::new(&buffer).render_label())
            } else {
                None
            }
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_bytes_filter_map(raw: &str) -> String {
    StrBytesFilterMapPayload::new(raw).unused_label()
}
