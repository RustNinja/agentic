use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscTryRecvUnwrapMapPayload {
    value: String,
}

impl MpscTryRecvUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-try-recv-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-try-recv-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-try-recv-unwrap-map:{}", self.value)
    }
}

pub fn selected_mpsc_try_recv_unwrap_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscTryRecvUnwrapMapPayload>, Receiver<MpscTryRecvUnwrapMapPayload>) = mpsc::channel();
    tx.send(MpscTryRecvUnwrapMapPayload::new(raw)).ok();
    let label = rx.try_recv().unwrap().render_label();
    label
}

pub fn dead_live_mpsc_try_recv_unwrap_map(raw: &str) -> String {
    MpscTryRecvUnwrapMapPayload::new(raw).unused_label()
}
