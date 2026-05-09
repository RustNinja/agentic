pub struct StrLinesFilterMapPayload {
    value: String,
}

impl StrLinesFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-lines-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-lines-filter-map:{}", self.value)
    }
}

pub fn selected_str_lines_filter_map(raw: &str) -> String {
    raw.lines()
        .filter_map(|line| {
            (!line.trim().is_empty()).then(|| StrLinesFilterMapPayload::new(line).render_label())
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_str_lines_filter_map(raw: &str) -> String {
    StrLinesFilterMapPayload::new(raw).unused_label()
}
