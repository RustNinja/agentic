pub struct StrTrimEndMatchesMapPayload {
    value: String,
}

impl StrTrimEndMatchesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-trim-end-matches-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-trim-end-matches-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-trim-end-matches-map:{}", self.value)
    }
}

pub fn selected_str_trim_end_matches_map(raw: &str) -> String {
    let segment = raw.trim_end_matches('z');
    StrTrimEndMatchesMapPayload::new(segment).render_label()
}

pub fn dead_live_str_trim_end_matches_map(raw: &str) -> String {
    StrTrimEndMatchesMapPayload::new(raw).unused_label()
}
