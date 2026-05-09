pub struct StrTrimMatchesMapPayload {
    value: String,
}

impl StrTrimMatchesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-trim-matches-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-trim-matches-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-trim-matches-map:{}", self.value)
    }
}

pub fn selected_str_trim_matches_map(raw: &str) -> String {
    std::iter::once(raw.trim_matches('/'))
        .map(|part| StrTrimMatchesMapPayload::new(part).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_trim_matches_map(raw: &str) -> String {
    StrTrimMatchesMapPayload::new(raw).unused_label()
}
