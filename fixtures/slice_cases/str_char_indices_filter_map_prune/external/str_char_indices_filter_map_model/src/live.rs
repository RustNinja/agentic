pub struct StrCharIndicesFilterMapPayload {
    value: String,
}

impl StrCharIndicesFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-char-indices-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-char-indices-filter-map:{}", self.value)
    }
}

pub fn selected_str_char_indices_filter_map(raw: &str) -> String {
    raw.char_indices()
        .filter_map(|(index, ch)| {
            ch.is_alphabetic().then(|| {
                StrCharIndicesFilterMapPayload::new(&format!("{index}:{ch}")).render_label()
            })
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_str_char_indices_filter_map(raw: &str) -> String {
    StrCharIndicesFilterMapPayload::new(raw).unused_label()
}
