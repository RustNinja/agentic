use codex_protocol_codegen::selected_wire_event;

pub struct CoreGeneratedEvent {
    label: String,
}

impl CoreGeneratedEvent {
    pub fn new(label: &str) -> Self {
        Self {
            label: normalize_core_label(label),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn dead_summary(&self) -> String {
        format!("dead-core-event:{}", self.label)
    }
}

pub fn render_core_generated_event(event: &CoreGeneratedEvent) -> String {
    let wire = selected_wire_event(event.label());
    format!("core:{wire}")
}

fn normalize_core_label(label: &str) -> String {
    label.trim().to_ascii_lowercase()
}

pub fn dead_live_core_codegen(label: &str) -> String {
    CoreGeneratedEvent::new(label).dead_summary()
}

