pub struct StrCharsFilterMapPayload {
    value: String,
}

impl StrCharsFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-chars-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-chars-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-chars-filter-map:{}", self.value)
    }
}

pub fn selected_str_chars_filter_map(raw: &str) -> String {
    raw.chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphabetic() {
                let buffer = ch.to_string();
                Some(StrCharsFilterMapPayload::new(&buffer).render_label())
            } else {
                None
            }
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_chars_filter_map(raw: &str) -> String {
    StrCharsFilterMapPayload::new(raw).unused_label()
}
