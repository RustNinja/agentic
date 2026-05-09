use std::collections::BinaryHeap;
pub struct BinaryheapPushPeekMapPayload {
    value: String,
}

impl BinaryheapPushPeekMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-push-peek-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-push-peek-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-push-peek-map:{}", self.value)
    }
}

pub fn selected_binaryheap_push_peek_map(raw: &str) -> String {
    let mut values = BinaryHeap::new();
    values.push(raw.to_string());
    values
        .peek()
        .map(|value| BinaryheapPushPeekMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_binaryheap_push_peek_map(raw: &str) -> String {
    BinaryheapPushPeekMapPayload::new(raw).unused_label()
}
