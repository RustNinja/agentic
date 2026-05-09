pub struct StrMatchIndicesMapPayload {
    value: String,
}

impl StrMatchIndicesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-match-indices-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-match-indices-map:{}", self.value)
    }
}

pub fn selected_str_match_indices_map(raw: &str) -> String {
    raw.match_indices("live")
        .map(|(index, part)| {
            StrMatchIndicesMapPayload::new(&format!("{index}:{part}")).render_label()
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_str_match_indices_map(raw: &str) -> String {
    StrMatchIndicesMapPayload::new(raw).unused_label()
}
