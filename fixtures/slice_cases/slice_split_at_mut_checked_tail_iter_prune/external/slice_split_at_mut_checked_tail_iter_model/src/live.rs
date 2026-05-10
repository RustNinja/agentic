pub fn selected_slice_split_at_mut_checked_tail_iter(raw: &str) -> String {
    let mut payloads = slice_split_at_mut_checked_tail_iter_items(raw);
    payloads
        .split_at_mut_checked(1)
        .map(|(_head, tail)| {
            tail.iter_mut()
                .map(|payload| payload.bump_and_render())
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceSplitAtMutCheckedTailIterPayload {
    value: String,
}

impl SliceSplitAtMutCheckedTailIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_split_at_mut_checked_tail_iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_split_at_mut_checked_tail_iter:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-at-mut-checked-tail-iter:{}", self.value)
    }
}

fn slice_split_at_mut_checked_tail_iter_items(raw: &str) -> Vec<SliceSplitAtMutCheckedTailIterPayload> {
    vec![
        SliceSplitAtMutCheckedTailIterPayload::new(raw),
        SliceSplitAtMutCheckedTailIterPayload::new("middle"),
        SliceSplitAtMutCheckedTailIterPayload::new("tail"),
    ]
}

pub fn dead_live_slice_split_at_mut_checked_tail_iter(raw: &str) -> String {
    SliceSplitAtMutCheckedTailIterPayload::new(raw).unused_label()
}
