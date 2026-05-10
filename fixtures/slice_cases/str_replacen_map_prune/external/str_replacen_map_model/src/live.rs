pub struct StrReplacenMapPayload {
    value: String,
}

impl StrReplacenMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-replacen-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-replacen-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-replacen-map:{}", self.value)
    }
}

pub fn selected_str_replacen_map(raw: &str) -> String {
    let value = raw.replacen('a', "b", 1);
    StrReplacenMapPayload::new(&value).render_label()
}

pub fn dead_live_str_replacen_map(raw: &str) -> String {
    StrReplacenMapPayload::new(raw).unused_label()
}
