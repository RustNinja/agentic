pub fn selected_slice_split_first_chunk_mut_map(raw: &str) -> String {
    let mut payloads = slice_split_first_chunk_mut_map_items(raw);
    payloads
        .split_first_chunk_mut::<2>()
        .map(|([head, _second], tail)| format!("{}:{}", head.bump_and_render(), tail.len()))
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceSplitFirstChunkMutMapPayload {
    value: String,
}

impl SliceSplitFirstChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_split_first_chunk_mut_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_split_first_chunk_mut_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-first-chunk-mut-map:{}", self.value)
    }
}

fn slice_split_first_chunk_mut_map_items(raw: &str) -> Vec<SliceSplitFirstChunkMutMapPayload> {
    vec![
        SliceSplitFirstChunkMutMapPayload::new(raw),
        SliceSplitFirstChunkMutMapPayload::new("middle"),
        SliceSplitFirstChunkMutMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_split_first_chunk_mut_map(raw: &str) -> String {
    SliceSplitFirstChunkMutMapPayload::new(raw).unused_label()
}
