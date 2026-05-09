#[derive(Clone)]
pub struct SliceSplitAtMutTailIterPayload {
    value: String,
}

impl SliceSplitAtMutTailIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice-split-at-mut-tail-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("slice-split-at-mut-tail-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-at-mut-tail-iter:{}", self.value)
    }
}

pub fn selected_slice_split_at_mut_tail_iter(raw: &str) -> String {
    let mut payloads = [
        SliceSplitAtMutTailIterPayload::new("head"),
        SliceSplitAtMutTailIterPayload::new(raw),
        SliceSplitAtMutTailIterPayload::new("tail"),
    ];
    let (_head, tail) = payloads.split_at_mut(1);
    tail.iter_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "slice-split-at-mut-tail-iter:missing".to_string())
}

pub fn dead_live_slice_split_at_mut_tail_iter(raw: &str) -> String {
    SliceSplitAtMutTailIterPayload::new(raw).dead_method()
}
