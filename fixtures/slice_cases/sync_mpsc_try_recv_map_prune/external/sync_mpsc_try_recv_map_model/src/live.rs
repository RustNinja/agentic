use std::sync::mpsc::{self, Receiver, SyncSender};
pub struct SyncMpscTryRecvMapPayload {
    value: String,
}

impl SyncMpscTryRecvMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("sync-mpsc-try-recv-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("sync-mpsc-try-recv-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-sync-mpsc-try-recv-map:{}", self.value)
    }
}

pub fn selected_sync_mpsc_try_recv_map(raw: &str) -> String {
    let (tx, rx): (SyncSender<SyncMpscTryRecvMapPayload>, Receiver<SyncMpscTryRecvMapPayload>) = mpsc::sync_channel(1);
    tx.send(SyncMpscTryRecvMapPayload::new(raw)).ok();
    rx.try_recv()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_sync_mpsc_try_recv_map(raw: &str) -> String {
    SyncMpscTryRecvMapPayload::new(raw).unused_label()
}
