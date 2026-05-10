use codex_core::{render_core_generated_event, CoreGeneratedEvent};
use opensourced::opensourced;

#[opensourced]
pub fn render_generated_event(label: &str) -> String {
    let event = CoreGeneratedEvent::new(label);
    format!("ui:{}", render_core_generated_event(&event))
}

pub fn dead_generated_event(label: &str) -> String {
    codex_core::dead_core_codegen(label)
}

