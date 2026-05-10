pub fn selected_slice_split_at_checked_tail_iter(raw: &str) -> String {
    let payloads = slice_split_at_checked_tail_iter_items(raw);
    payloads
        .split_at_checked(1)
        .map(|(_head, tail)| {
            tail.iter()
                .map(|payload| payload.render_label())
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_else(|| "empty".to_string())
}

pub struct SliceSplitAtCheckedTailIterPayload {
    value: String,
}

impl SliceSplitAtCheckedTailIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("slice_split_at_checked_tail_iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("slice_split_at_checked_tail_iter:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-slice-split-at-checked-tail-iter:{}", self.value)
    }
}

fn slice_split_at_checked_tail_iter_items(raw: &str) -> Vec<SliceSplitAtCheckedTailIterPayload> {
    vec![
        SliceSplitAtCheckedTailIterPayload::new(raw),
        SliceSplitAtCheckedTailIterPayload::new("middle"),
        SliceSplitAtCheckedTailIterPayload::new("tail"),
    ]
}

pub fn dead_live_slice_split_at_checked_tail_iter(raw: &str) -> String {
    SliceSplitAtCheckedTailIterPayload::new(raw).unused_label()
}
