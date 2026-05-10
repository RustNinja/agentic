pub struct DeadIterRepeatNMapItem;

pub struct DeadIterRepeatNMapPayload {
    value: String,
}

impl DeadIterRepeatNMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iter-repeat-n-map:{}", self.value)
    }
}

pub fn dead_iter_repeat_n_map(raw: &str) -> String {
    DeadIterRepeatNMapPayload::new(raw).dead_method()
}
