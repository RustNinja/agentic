pub fn selected_slice_as_rchunks_iter(raw: &str) -> String {
    let payloads = slice_as_rchunks_iter_items(raw);
    let (_remainder, chunks) = payloads.as_rchunks::<2>();
    chunks
        .iter()
        .map(|[head, _tail]| head.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub struct SliceAsRchunksIterPayload {
    value: String,
}

impl SliceAsRchunksIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_as_rchunks_iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_as_rchunks_iter:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-as-rchunks-iter:{}", self.value)
    }
}

fn slice_as_rchunks_iter_items(raw: &str) -> Vec<SliceAsRchunksIterPayload> {
    vec![
        SliceAsRchunksIterPayload::new(raw),
        SliceAsRchunksIterPayload::new("middle"),
        SliceAsRchunksIterPayload::new("tail"),
    ]
}

pub fn dead_live_slice_as_rchunks_iter(raw: &str) -> String {
    SliceAsRchunksIterPayload::new(raw).unused_label()
}
