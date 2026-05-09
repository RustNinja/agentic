pub struct StrRmatchIndicesMapPayload {
    value: String,
}

impl StrRmatchIndicesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("str-rmatch-indices-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("str-rmatch-indices-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-str-rmatch-indices-map:{}", self.value)
    }
}

pub fn selected_str_rmatch_indices_map(raw: &str) -> String {
    raw.rmatch_indices('/')
        .map(|(idx, part)| {
            let buffer = format!("{idx}:{part}");
            StrRmatchIndicesMapPayload::new(&buffer).render_label()
        })
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_str_rmatch_indices_map(raw: &str) -> String {
    StrRmatchIndicesMapPayload::new(raw).unused_label()
}
