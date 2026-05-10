pub struct IteratorTryRfoldResultPayload {
    value: String,
}

impl IteratorTryRfoldResultPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-try-rfold-result:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-try-rfold-result:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-try-rfold-result:{}", self.value)
    }
}

fn iterator_try_rfold_result_items(raw: &str) -> Vec<IteratorTryRfoldResultPayload> {
    vec![IteratorTryRfoldResultPayload::new(raw), IteratorTryRfoldResultPayload::new("tail")]
}

pub fn selected_iterator_try_rfold_result(raw: &str) -> String {
    iterator_try_rfold_result_items(raw)
        .into_iter()
        .try_rfold(String::new(), |mut acc, payload| -> Result<String, String> {
            acc.push_str(&payload.render_label());
            Ok(acc)
        })
        .unwrap_or_default()
}

pub fn dead_live_iterator_try_rfold_result(raw: &str) -> String {
    IteratorTryRfoldResultPayload::new(raw).unused_label()
}
