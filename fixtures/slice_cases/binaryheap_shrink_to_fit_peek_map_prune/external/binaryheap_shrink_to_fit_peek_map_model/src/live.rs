use std::collections::BinaryHeap;
pub struct BinaryheapShrinkToFitPeekMapPayload {
    value: String,
}

impl BinaryheapShrinkToFitPeekMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-shrink-to-fit-peek-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-shrink-to-fit-peek-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-shrink-to-fit-peek-map:{}", self.value)
    }
}

pub fn selected_binaryheap_shrink_to_fit_peek_map(raw: &str) -> String {
    let mut values = BinaryHeap::from([raw.to_string()]);
    values.shrink_to_fit();
    values
        .peek()
        .map(|value| BinaryheapShrinkToFitPeekMapPayload::new(value).render_label())
        .unwrap_or_default()
}

pub fn dead_live_binaryheap_shrink_to_fit_peek_map(raw: &str) -> String {
    BinaryheapShrinkToFitPeekMapPayload::new(raw).unused_label()
}
