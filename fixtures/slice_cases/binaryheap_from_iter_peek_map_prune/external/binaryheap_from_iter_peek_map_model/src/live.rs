use std::collections::BinaryHeap;
pub struct BinaryheapFromIterPeekMapPayload {
    value: String,
}

impl BinaryheapFromIterPeekMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-from-iter-peek-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-from-iter-peek-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-from-iter-peek-map:{}", self.value)
    }
}

pub fn selected_binaryheap_from_iter_peek_map(raw: &str) -> String {
    let values = BinaryHeap::from([raw.to_string()]);
    values
        .peek()
        .map(|value| BinaryheapFromIterPeekMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_binaryheap_from_iter_peek_map(raw: &str) -> String {
    BinaryheapFromIterPeekMapPayload::new(raw).unused_label()
}
