pub struct DeadMpscIterNextMapItem;

pub struct DeadMpscIterNextMapPayload {
    value: String,
}

impl DeadMpscIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-iter-next-map:{}", self.value)
    }
}

pub fn dead_mpsc_iter_next_map(raw: &str) -> String {
    DeadMpscIterNextMapPayload::new(raw).dead_method()
}
