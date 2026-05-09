pub struct StrMatchesMapPayload {
    value: String,
}

impl StrMatchesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-matches-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-matches-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-matches-map:{}", self.value)
    }
}

pub fn selected_str_matches_map(raw: &str) -> String {
    raw.matches('a')
        .map(StrMatchesMapPayload::new)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_matches_map(raw: &str) -> String {
    StrMatchesMapPayload::new(raw).unused_label()
}
