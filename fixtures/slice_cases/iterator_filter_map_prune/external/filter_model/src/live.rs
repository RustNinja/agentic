pub struct FilterEntry {
    value: String,
}

impl FilterEntry {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_if_live(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| format!("filter:{}", self.value))
    }

    pub fn dead_method(&self) -> String {
        format!("dead-filter:{}", self.value)
    }
}

fn build_entries(raw: &str) -> Vec<FilterEntry> {
    raw.split(',').map(FilterEntry::new).collect()
}

pub fn selected_filter(raw: &str) -> String {
    build_entries(raw)
        .iter()
        .filter_map(|entry| entry.render_if_live())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_filter(raw: &str) -> String {
    FilterEntry::new(raw).dead_method()
}
