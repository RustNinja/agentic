pub struct StrBytesEnumerateMapPayload {
    value: String,
}

impl StrBytesEnumerateMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-bytes-enumerate-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-bytes-enumerate-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-bytes-enumerate-map:{}", self.value)
    }
}

pub fn selected_str_bytes_enumerate_map(raw: &str) -> String {
    raw.bytes()
        .enumerate()
        .map(|(idx, byte)| {
            StrBytesEnumerateMapPayload::new(&format!("{idx}:{byte}")).render_label()
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_bytes_enumerate_map(raw: &str) -> String {
    StrBytesEnumerateMapPayload::new(raw).unused_label()
}
