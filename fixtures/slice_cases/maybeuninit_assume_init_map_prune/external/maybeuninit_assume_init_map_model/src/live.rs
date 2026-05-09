use std::mem::MaybeUninit;

#[derive(Clone)]
pub struct MaybeUninitAssumeInitMapPayload {
    value: String,
}

impl MaybeUninitAssumeInitMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("maybeuninit-assume-init-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("maybeuninit-assume-init-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-maybeuninit-assume-init-map:{}", self.value)
    }
}

pub fn selected_maybeuninit_assume_init_map(raw: &str) -> String {
    let payload: MaybeUninit<MaybeUninitAssumeInitMapPayload> =
        MaybeUninit::new(MaybeUninitAssumeInitMapPayload::new(raw));
    unsafe { payload.assume_init().render_label() }
}

pub fn dead_live_maybeuninit_assume_init_map(raw: &str) -> String {
    MaybeUninitAssumeInitMapPayload::new(raw).dead_method()
}
