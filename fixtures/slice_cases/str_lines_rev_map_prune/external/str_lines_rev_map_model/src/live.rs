pub struct StrLinesRevMapPayload {
    value: String,
}

impl StrLinesRevMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-lines-rev-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-lines-rev-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-lines-rev-map:{}", self.value)
    }
}

pub fn selected_str_lines_rev_map(raw: &str) -> String {
    raw.lines()
        .rev()
        .map(StrLinesRevMapPayload::new)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_lines_rev_map(raw: &str) -> String {
    StrLinesRevMapPayload::new(raw).unused_label()
}
