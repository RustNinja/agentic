pub struct StrSplitnFindMapPayload {
    value: String,
}

impl StrSplitnFindMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("str-splitn-find-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-splitn-find-map:{}", self.value)
    }
}

pub fn selected_str_splitn_find_map(raw: &str) -> String {
    raw.splitn(3, ',')
        .map(StrSplitnFindMapPayload::new)
        .find_map(|payload| payload.maybe_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_splitn_find_map(raw: &str) -> String {
    StrSplitnFindMapPayload::new(raw).unused_label()
}
