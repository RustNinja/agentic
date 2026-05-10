mod generated {
    include!(concat!(env!("OUT_DIR"), "/litter_codegen_bindings.rs"));
}

pub struct GeneratedEvent {
    label: String,
    payload: String,
}

impl GeneratedEvent {
    pub fn new(label: &str, payload: &str) -> Self {
        Self {
            label: label.to_string(),
            payload: payload.to_string(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn render(&self) -> String {
        format!("{}={}", self.label(), self.payload())
    }

    pub fn dead_debug(&self) -> String {
        format!("dead-generated:{}", self.label)
    }
}

pub fn selected_wire_event(label: &str) -> String {
    generated::generated_event(label).render()
}

pub fn generated_event_helper(label: &str) -> GeneratedEvent {
    GeneratedEvent::new(label, &normalize_generated_payload(label))
}

fn normalize_generated_payload(label: &str) -> String {
    label.trim().replace(' ', "_")
}

pub fn dead_wire_event(label: &str) -> String {
    dead_generated_helper(label).dead_debug()
}

pub fn dead_generated_helper(label: &str) -> GeneratedEvent {
    GeneratedEvent::new(label, "dead")
}

