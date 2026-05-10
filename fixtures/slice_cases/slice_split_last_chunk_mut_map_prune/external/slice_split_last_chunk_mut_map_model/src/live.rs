pub fn selected_slice_split_last_chunk_mut_map(raw: &str) -> String {
    let mut payloads = slice_split_last_chunk_mut_map_items(raw);
    payloads
        .split_last_chunk_mut::<2>()
        .map(|(head, [tail, _last])| format!("{}:{}", tail.bump_and_render(), head.len()))
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceSplitLastChunkMutMapPayload {
    value: String,
}

impl SliceSplitLastChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_split_last_chunk_mut_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_split_last_chunk_mut_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-last-chunk-mut-map:{}", self.value)
    }
}

fn slice_split_last_chunk_mut_map_items(raw: &str) -> Vec<SliceSplitLastChunkMutMapPayload> {
    vec![
        SliceSplitLastChunkMutMapPayload::new(raw),
        SliceSplitLastChunkMutMapPayload::new("middle"),
        SliceSplitLastChunkMutMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_split_last_chunk_mut_map(raw: &str) -> String {
    SliceSplitLastChunkMutMapPayload::new(raw).unused_label()
}
