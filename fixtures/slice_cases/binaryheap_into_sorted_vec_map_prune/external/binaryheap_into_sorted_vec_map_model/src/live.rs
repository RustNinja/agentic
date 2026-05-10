use std::collections::BinaryHeap;
pub struct BinaryheapIntoSortedVecMapPayload {
    value: String,
}

impl BinaryheapIntoSortedVecMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-into-sorted-vec-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("binaryheap-into-sorted-vec-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-binaryheap-into-sorted-vec-map:{}", self.value)
    }
}

pub fn selected_binaryheap_into_sorted_vec_map(raw: &str) -> String {
    let values = BinaryHeap::from([raw.to_string()]);
    values
        .into_sorted_vec()
        .into_iter()
        .map(|value| BinaryheapIntoSortedVecMapPayload::new(&value).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_binaryheap_into_sorted_vec_map(raw: &str) -> String {
    BinaryheapIntoSortedVecMapPayload::new(raw).unused_label()
}
