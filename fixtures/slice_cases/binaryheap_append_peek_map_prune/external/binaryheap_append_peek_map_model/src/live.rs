use std::collections::BinaryHeap;
pub struct BinaryheapAppendPeekMapPayload {
    value: String,
}

impl BinaryheapAppendPeekMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-append-peek-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-append-peek-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-append-peek-map:{}", self.value)
    }
}

pub fn selected_binaryheap_append_peek_map(raw: &str) -> String {
    let mut values = BinaryHeap::from([raw.to_string()]);
    let mut extras = BinaryHeap::from([format!("{raw}-tail")]);
    values.append(&mut extras);
    values
        .peek()
        .map(|value| BinaryheapAppendPeekMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_binaryheap_append_peek_map(raw: &str) -> String {
    BinaryheapAppendPeekMapPayload::new(raw).unused_label()
}
