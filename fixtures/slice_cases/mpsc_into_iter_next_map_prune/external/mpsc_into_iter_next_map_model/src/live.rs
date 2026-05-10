use std::sync::mpsc::{self, Receiver, Sender};
pub struct MpscIntoIterNextMapPayload {
    value: String,
}

impl MpscIntoIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("mpsc-into-iter-next-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("mpsc-into-iter-next-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-mpsc-into-iter-next-map:{}", self.value)
    }
}

pub fn selected_mpsc_into_iter_next_map(raw: &str) -> String {
    let (tx, rx): (Sender<MpscIntoIterNextMapPayload>, Receiver<MpscIntoIterNextMapPayload>) = mpsc::channel();
    tx.send(MpscIntoIterNextMapPayload::new(raw)).ok();
    drop(tx);
    rx.into_iter()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_mpsc_into_iter_next_map(raw: &str) -> String {
    MpscIntoIterNextMapPayload::new(raw).unused_label()
}
