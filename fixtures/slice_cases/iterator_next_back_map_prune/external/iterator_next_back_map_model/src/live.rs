pub struct IteratorNextBackMapPayload {
    value: String,
}

impl IteratorNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-next-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-next-back-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-next-back-map:{}", self.value)
    }
}

fn iterator_next_back_map_items(raw: &str) -> Vec<IteratorNextBackMapPayload> {
    vec![IteratorNextBackMapPayload::new("head"), IteratorNextBackMapPayload::new(raw)]
}

pub fn selected_iterator_next_back_map(raw: &str) -> String {
    iterator_next_back_map_items(raw)
        .into_iter()
        .next_back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-next-back-map:missing".to_string())
}

pub fn dead_live_iterator_next_back_map(raw: &str) -> String {
    IteratorNextBackMapPayload::new(raw).unused_label()
}
