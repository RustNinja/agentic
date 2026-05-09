#[derive(Clone)]
pub struct SliceSplitAtTailIterPayload {
    value: String,
}

impl SliceSplitAtTailIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-at-tail-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-split-at-tail-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-at-tail-iter:{}", self.value)
    }
}

pub fn selected_slice_split_at_tail_iter(raw: &str) -> String {
    let payloads = [
        SliceSplitAtTailIterPayload::new("head"),
        SliceSplitAtTailIterPayload::new(raw),
        SliceSplitAtTailIterPayload::new("tail"),
    ];
    let (_head, tail) = payloads.split_at(1);
    tail.iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "slice-split-at-tail-iter:missing".to_string())
}

pub fn dead_live_slice_split_at_tail_iter(raw: &str) -> String {
    SliceSplitAtTailIterPayload::new(raw).dead_method()
}
