pub struct IteratorNthBackMapPayload {
    value: String,
}

impl IteratorNthBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-nth-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-nth-back-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-nth-back-map:{}", self.value)
    }
}

fn iterator_nth_back_map_items(raw: &str) -> Vec<IteratorNthBackMapPayload> {
    vec![IteratorNthBackMapPayload::new("head"), IteratorNthBackMapPayload::new(raw)]
}

pub fn selected_iterator_nth_back_map(raw: &str) -> String {
    iterator_nth_back_map_items(raw)
        .into_iter()
        .nth_back(0)
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-nth-back-map:missing".to_string())
}

pub fn dead_live_iterator_nth_back_map(raw: &str) -> String {
    IteratorNthBackMapPayload::new(raw).unused_label()
}
