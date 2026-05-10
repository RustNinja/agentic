pub struct IteratorRfoldPayload {
    value: String,
}

impl IteratorRfoldPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-rfold:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-rfold:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-rfold:{}", self.value)
    }
}

fn iterator_rfold_items(raw: &str) -> Vec<IteratorRfoldPayload> {
    vec![IteratorRfoldPayload::new(raw), IteratorRfoldPayload::new("tail")]
}

pub fn selected_iterator_rfold(raw: &str) -> String {
    iterator_rfold_items(raw)
        .into_iter()
        .rfold(String::new(), |mut acc, payload| {
            acc.push_str(&payload.render_label());
            acc
        })
}

pub fn dead_live_iterator_rfold(raw: &str) -> String {
    IteratorRfoldPayload::new(raw).unused_label()
}
