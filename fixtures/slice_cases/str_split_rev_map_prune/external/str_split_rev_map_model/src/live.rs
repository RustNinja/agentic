pub struct StrSplitRevMapPayload {
    value: String,
}

impl StrSplitRevMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-rev-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-split-rev-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-rev-map:{}", self.value)
    }
}

pub fn selected_str_split_rev_map(raw: &str) -> String {
    raw.split(':')
        .rev()
        .map(StrSplitRevMapPayload::new)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_split_rev_map(raw: &str) -> String {
    StrSplitRevMapPayload::new(raw).unused_label()
}
