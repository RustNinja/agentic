pub struct DeadMpscTryRecvMapItem;

pub struct DeadMpscTryRecvMapPayload {
    value: String,
}

impl DeadMpscTryRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-try-recv-map:{}", self.value)
    }
}

pub fn dead_mpsc_try_recv_map(raw: &str) -> String {
    DeadMpscTryRecvMapPayload::new(raw).dead_method()
}
