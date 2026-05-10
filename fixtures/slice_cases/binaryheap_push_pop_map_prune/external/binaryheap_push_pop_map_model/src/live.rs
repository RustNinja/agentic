use std::collections::BinaryHeap;
pub struct BinaryheapPushPopMapPayload {
    value: String,
}

impl BinaryheapPushPopMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-push-pop-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-push-pop-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-push-pop-map:{}", self.value)
    }
}

pub fn selected_binaryheap_push_pop_map(raw: &str) -> String {
    let mut values = BinaryHeap::new();
    values.push(raw.to_string());
    values
        .pop()
        .map(|value| BinaryheapPushPopMapPayload::new(&value).render_label())
        .unwrap_or_default()
}

pub fn dead_live_binaryheap_push_pop_map(raw: &str) -> String {
    BinaryheapPushPopMapPayload::new(raw).unused_label()
}
