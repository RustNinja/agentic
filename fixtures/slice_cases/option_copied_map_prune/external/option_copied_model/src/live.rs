#[derive(Clone, Copy)]
pub struct OptionCopiedMapPayload {
    value: i32,
}

impl OptionCopiedMapPayload {
    pub const fn new_const(value: i32) -> Self {
        Self { value }
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().len() as i32,
        }
    }

    pub fn render_label(&self) -> String {
        format!("option-copied-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value += 1;
        format!("option-copied-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-copied-map:{}", self.value)
    }
}

const LIVE_OPTIONCOPIEDMAP_PAYLOAD: OptionCopiedMapPayload = OptionCopiedMapPayload::new_const(7);

fn option_copied_map_payload(raw: &str) -> Option<&'static OptionCopiedMapPayload> {
    if raw.trim().is_empty() {
        None
    } else {
        Some(&LIVE_OPTIONCOPIEDMAP_PAYLOAD)
    }
}

pub fn selected_option_copied_map(raw: &str) -> String {
    option_copied_map_payload(raw)
        .copied()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "option-copied-map:missing".to_string())
}

pub fn dead_live_option_copied_map(raw: &str) -> String {
    OptionCopiedMapPayload::new(raw).dead_method()
}
