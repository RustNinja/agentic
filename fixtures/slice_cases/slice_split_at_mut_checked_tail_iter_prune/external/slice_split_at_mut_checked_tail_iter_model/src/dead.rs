pub struct DeadSliceSplitAtMutCheckedTailIterItem;

pub struct DeadSliceSplitAtMutCheckedTailIterPayload {
    value: String,
}

impl DeadSliceSplitAtMutCheckedTailIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-at-mut-checked-tail-iter:{}", self.value)
    }
}

pub fn dead_slice_split_at_mut_checked_tail_iter(raw: &str) -> String {
    DeadSliceSplitAtMutCheckedTailIterPayload::new(raw).dead_method()
}
