pub fn selected_slice_as_rchunks_mut_iter(raw: &str) -> String {
    let mut payloads = slice_as_rchunks_mut_iter_items(raw);
    let (_remainder, chunks) = payloads.as_rchunks_mut::<2>();
    chunks
        .iter_mut()
        .map(|[head, _tail]| head.bump_and_render())
        .collect::<Vec<_>>()
        .join("|")
}

pub struct SliceAsRchunksMutIterPayload {
    value: String,
}

impl SliceAsRchunksMutIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_as_rchunks_mut_iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_as_rchunks_mut_iter:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-as-rchunks-mut-iter:{}", self.value)
    }
}

fn slice_as_rchunks_mut_iter_items(raw: &str) -> Vec<SliceAsRchunksMutIterPayload> {
    vec![
        SliceAsRchunksMutIterPayload::new(raw),
        SliceAsRchunksMutIterPayload::new("middle"),
        SliceAsRchunksMutIterPayload::new("tail"),
    ]
}

pub fn dead_live_slice_as_rchunks_mut_iter(raw: &str) -> String {
    SliceAsRchunksMutIterPayload::new(raw).unused_label()
}
