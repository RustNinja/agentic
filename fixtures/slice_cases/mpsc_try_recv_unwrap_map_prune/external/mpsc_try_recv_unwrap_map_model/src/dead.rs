pub struct DeadMpscTryRecvUnwrapMapItem;

pub struct DeadMpscTryRecvUnwrapMapPayload {
    value: String,
}

impl DeadMpscTryRecvUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-mpsc-try-recv-unwrap-map:{}", self.value)
    }
}

pub fn dead_mpsc_try_recv_unwrap_map(raw: &str) -> String {
    DeadMpscTryRecvUnwrapMapPayload::new(raw).dead_method()
}
