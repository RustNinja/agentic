pub fn selected_slice_split_last_chunk_map(raw: &str) -> String {
    let payloads = slice_split_last_chunk_map_items(raw);
    payloads
        .split_last_chunk::<2>()
        .map(|(head, [tail, _last])| format!("{}:{}", tail.render_label(), head.len()))
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceSplitLastChunkMapPayload {
    value: String,
}

impl SliceSplitLastChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_split_last_chunk_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_split_last_chunk_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-last-chunk-map:{}", self.value)
    }
}

fn slice_split_last_chunk_map_items(raw: &str) -> Vec<SliceSplitLastChunkMapPayload> {
    vec![
        SliceSplitLastChunkMapPayload::new(raw),
        SliceSplitLastChunkMapPayload::new("middle"),
        SliceSplitLastChunkMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_split_last_chunk_map(raw: &str) -> String {
    SliceSplitLastChunkMapPayload::new(raw).unused_label()
}
