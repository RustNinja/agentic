use std::mem::MaybeUninit;
pub struct MaybeuninitWriteMapPayload {
    value: String,
}

impl MaybeuninitWriteMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("maybeuninit-write-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("maybeuninit-write-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-maybeuninit-write-map:{}", self.value)
    }
}

pub fn selected_maybeuninit_write_map(raw: &str) -> String {
    let mut slot = MaybeUninit::uninit();
    let payload = slot.write(MaybeuninitWriteMapPayload::new(raw));
    payload.bump_and_render()
}

pub fn dead_live_maybeuninit_write_map(raw: &str) -> String {
    MaybeuninitWriteMapPayload::new(raw).unused_label()
}
