pub struct DeadSliceSplitAtCheckedTailIterItem;

pub struct DeadSliceSplitAtCheckedTailIterPayload {
    value: String,
}

impl DeadSliceSplitAtCheckedTailIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-at-checked-tail-iter:{}", self.value)
    }
}

pub fn dead_slice_split_at_checked_tail_iter(raw: &str) -> String {
    DeadSliceSplitAtCheckedTailIterPayload::new(raw).dead_method()
}
