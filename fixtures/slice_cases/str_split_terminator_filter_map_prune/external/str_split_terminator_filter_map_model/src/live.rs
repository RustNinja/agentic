pub struct StrSplitTerminatorFilterMapPayload {
    value: String,
}

impl StrSplitTerminatorFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-terminator-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-terminator-filter-map:{}", self.value)
    }
}

pub fn selected_str_split_terminator_filter_map(raw: &str) -> String {
    raw.split_terminator(';')
        .filter_map(|part| {
            (!part.trim().is_empty())
                .then(|| StrSplitTerminatorFilterMapPayload::new(part).render_label())
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_str_split_terminator_filter_map(raw: &str) -> String {
    StrSplitTerminatorFilterMapPayload::new(raw).unused_label()
}
