pub struct StrSplitInclusiveRevMapPayload {
    value: String,
}

impl StrSplitInclusiveRevMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-inclusive-rev-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-split-inclusive-rev-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-inclusive-rev-map:{}", self.value)
    }
}

pub fn selected_str_split_inclusive_rev_map(raw: &str) -> String {
    raw.split_inclusive(':')
        .rev()
        .map(StrSplitInclusiveRevMapPayload::new)
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_split_inclusive_rev_map(raw: &str) -> String {
    StrSplitInclusiveRevMapPayload::new(raw).unused_label()
}
