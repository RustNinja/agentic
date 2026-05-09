pub struct StrEscapeDebugFlatMapPayload {
    value: String,
}

impl StrEscapeDebugFlatMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-escape-debug-flat-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-escape-debug-flat-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-escape-debug-flat-map:{}", self.value)
    }
}

pub fn selected_str_escape_debug_flat_map(raw: &str) -> String {
    raw.chars()
        .flat_map(char::escape_debug)
        .map(|ch| StrEscapeDebugFlatMapPayload::new(&ch.to_string()).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_escape_debug_flat_map(raw: &str) -> String {
    StrEscapeDebugFlatMapPayload::new(raw).unused_label()
}
