pub struct DeadSyncMpscTryRecvMapItem;

pub struct DeadSyncMpscTryRecvMapPayload {
    value: String,
}

impl DeadSyncMpscTryRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-sync-mpsc-try-recv-map:{}", self.value)
    }
}

pub fn dead_sync_mpsc_try_recv_map(raw: &str) -> String {
    DeadSyncMpscTryRecvMapPayload::new(raw).dead_method()
}
