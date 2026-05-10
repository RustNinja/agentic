use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscRecvUnwrapMapPayload {
    value: String,
}

impl MpscRecvUnwrapMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-recv-unwrap-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-recv-unwrap-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-recv-unwrap-map:{}", self.value)
    }
}

pub fn selected_mpsc_recv_unwrap_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscRecvUnwrapMapPayload>, Receiver<MpscRecvUnwrapMapPayload>) = mpsc::channel();
    tx.send(MpscRecvUnwrapMapPayload::new(raw)).ok();
    let label = rx.recv().unwrap().render_label();
    label
}

pub fn dead_live_mpsc_recv_unwrap_map(raw: &str) -> String {
    MpscRecvUnwrapMapPayload::new(raw).unused_label()
}
