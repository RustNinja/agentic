pub fn selected_slice_split_first_chunk_map(raw: &str) -> String {
    let payloads = slice_split_first_chunk_map_items(raw);
    payloads
        .split_first_chunk::<2>()
        .map(|([head, _second], tail)| format!("{}:{}", head.render_label(), tail.len()))
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceSplitFirstChunkMapPayload {
    value: String,
}

impl SliceSplitFirstChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_split_first_chunk_map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_split_first_chunk_map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-first-chunk-map:{}", self.value)
    }
}

fn slice_split_first_chunk_map_items(raw: &str) -> Vec<SliceSplitFirstChunkMapPayload> {
    vec![
        SliceSplitFirstChunkMapPayload::new(raw),
        SliceSplitFirstChunkMapPayload::new("middle"),
        SliceSplitFirstChunkMapPayload::new("tail"),
    ]
}

pub fn dead_live_slice_split_first_chunk_map(raw: &str) -> String {
    SliceSplitFirstChunkMapPayload::new(raw).unused_label()
}
