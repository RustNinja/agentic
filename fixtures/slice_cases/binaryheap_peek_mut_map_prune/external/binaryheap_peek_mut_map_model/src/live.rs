use std::collections::BinaryHeap;
pub struct BinaryheapPeekMutMapPayload {
    value: String,
}

impl BinaryheapPeekMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-peek-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-peek-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-peek-mut-map:{}", self.value)
    }
}

pub fn selected_binaryheap_peek_mut_map(raw: &str) -> String {
    let mut values = BinaryHeap::from([raw.to_string()]);
    values
        .peek_mut()
        .map(|mut value| {
            value.push_str("-top");
            BinaryheapPeekMutMapPayload::new(value.as_str()).render_label()
        })
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_binaryheap_peek_mut_map(raw: &str) -> String {
    BinaryheapPeekMutMapPayload::new(raw).unused_label()
}
