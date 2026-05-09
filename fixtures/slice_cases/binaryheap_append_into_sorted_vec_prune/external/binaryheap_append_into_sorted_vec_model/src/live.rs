use std::collections::BinaryHeap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BinaryheapAppendIntoSortedVecPayload {
    value: String,
}

impl BinaryheapAppendIntoSortedVecPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("binaryheap-append-into-sorted-vec:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("binaryheap-append-into-sorted-vec:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-append-into-sorted-vec:{}", self.value)
    }
}

fn binaryheap_append_into_sorted_vec_heap(
    raw: &str,
) -> BinaryHeap<BinaryheapAppendIntoSortedVecPayload> {
    let mut payloads = BinaryHeap::new();
    payloads.push(BinaryheapAppendIntoSortedVecPayload::new(raw));
    payloads.push(BinaryheapAppendIntoSortedVecPayload::new("tail"));
    payloads
}

pub fn selected_binaryheap_append_into_sorted_vec(raw: &str) -> String {
    let mut payloads = binaryheap_append_into_sorted_vec_heap(raw);
    let mut extras = binaryheap_append_into_sorted_vec_heap("extra");
    payloads.append(&mut extras);
    payloads
        .into_sorted_vec()
        .into_iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "binaryheap-append-into-sorted-vec:missing".to_string())
}

pub fn dead_live_binaryheap_append_into_sorted_vec(raw: &str) -> String {
    BinaryheapAppendIntoSortedVecPayload::new(raw).dead_method()
}
