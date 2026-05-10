use std::{sync::mpsc::{self, Receiver, Sender}, time::Duration};
pub struct MpscRecvTimeoutMapPayload {
    value: String,
}

impl MpscRecvTimeoutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-recv-timeout-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-recv-timeout-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-recv-timeout-map:{}", self.value)
    }
}

pub fn selected_mpsc_recv_timeout_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscRecvTimeoutMapPayload>, Receiver<MpscRecvTimeoutMapPayload>) = mpsc::channel();
    tx.send(MpscRecvTimeoutMapPayload::new(raw)).ok();
    rx.recv_timeout(Duration::from_millis(1))
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mpsc_recv_timeout_map(raw: &str) -> String {
    MpscRecvTimeoutMapPayload::new(raw).unused_label()
}
