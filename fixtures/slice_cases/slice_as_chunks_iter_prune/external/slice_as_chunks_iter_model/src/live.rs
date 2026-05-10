pub fn selected_slice_as_chunks_iter(raw: &str) -> String {
    let payloads = slice_as_chunks_iter_items(raw);
    let (chunks, _remainder) = payloads.as_chunks::<2>();
    chunks
        .iter()
        .map(|[head, _tail]| head.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub struct SliceAsChunksIterPayload {
    value: String,
}

impl SliceAsChunksIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_as_chunks_iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_as_chunks_iter:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-as-chunks-iter:{}", self.value)
    }
}

fn slice_as_chunks_iter_items(raw: &str) -> Vec<SliceAsChunksIterPayload> {
    vec![
        SliceAsChunksIterPayload::new(raw),
        SliceAsChunksIterPayload::new("middle"),
        SliceAsChunksIterPayload::new("tail"),
    ]
}

pub fn dead_live_slice_as_chunks_iter(raw: &str) -> String {
    SliceAsChunksIterPayload::new(raw).unused_label()
}
