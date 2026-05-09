pub struct StrSplitInclusiveFilterMapPayload {
    value: String,
}

impl StrSplitInclusiveFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-split-inclusive-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-split-inclusive-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-split-inclusive-filter-map:{}", self.value)
    }
}

pub fn selected_str_split_inclusive_filter_map(raw: &str) -> String {
    raw.split_inclusive(';')
        .filter_map(|part| {
            (!part.trim().is_empty())
                .then(|| StrSplitInclusiveFilterMapPayload::new(part).render_label())
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_split_inclusive_filter_map(raw: &str) -> String {
    StrSplitInclusiveFilterMapPayload::new(raw).unused_label()
}
