pub struct StrRsplitMapPayload {
    value: String,
}

impl StrRsplitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-rsplit-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-rsplit-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-rsplit-map:{}", self.value)
    }
}

pub fn selected_str_rsplit_map(raw: &str) -> String {
    raw.rsplit(':')
        .map(StrRsplitMapPayload::new)
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_rsplit_map(raw: &str) -> String {
    StrRsplitMapPayload::new(raw).unused_label()
}
