pub struct StrRsplitnFindMapPayload {
    value: String,
}

impl StrRsplitnFindMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-rsplitn-find-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-rsplitn-find-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-rsplitn-find-map:{}", self.value)
    }
}

pub fn selected_str_rsplitn_find_map(raw: &str) -> String {
    raw.rsplitn(3, ':')
        .find_map(|part| {
            (!part.is_empty()).then(|| StrRsplitnFindMapPayload::new(part).render_label())
        })
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_rsplitn_find_map(raw: &str) -> String {
    StrRsplitnFindMapPayload::new(raw).unused_label()
}
