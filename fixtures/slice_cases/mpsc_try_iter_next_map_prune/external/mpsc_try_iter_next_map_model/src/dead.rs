pub struct DeadMpscTryIterNextMapItem;

pub struct DeadMpscTryIterNextMapPayload {
    value: String,
}

impl DeadMpscTryIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-try-iter-next-map:{}", self.value)
    }
}

pub fn dead_mpsc_try_iter_next_map(raw: &str) -> String {
    DeadMpscTryIterNextMapPayload::new(raw).dead_method()
}
