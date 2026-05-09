pub struct StrCharEscapeDefaultMapPayload {
    value: String,
}

impl StrCharEscapeDefaultMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-char-escape-default-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-char-escape-default-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-char-escape-default-map:{}", self.value)
    }
}

pub fn selected_str_char_escape_default_map(raw: &str) -> String {
    raw.chars()
        .flat_map(char::escape_default)
        .map(|ch| {
            let buffer = ch.to_string();
            StrCharEscapeDefaultMapPayload::new(&buffer).render_label()
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_char_escape_default_map(raw: &str) -> String {
    StrCharEscapeDefaultMapPayload::new(raw).unused_label()
}
