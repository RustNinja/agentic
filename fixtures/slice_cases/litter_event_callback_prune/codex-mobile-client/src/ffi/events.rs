use event_api::{EventCallbackHandle, EventRegistration, EventSnapshotDto};
use opensourced::opensourced;

#[opensourced]
pub fn register_event_callback(label: &str) -> EventSnapshotDto {
    let registration = EventRegistration::new(label);
    let handle = EventCallbackHandle::register(registration);
    handle.emit_preview("connected")
}

pub fn dead_event_callback(label: &str) -> String {
    EventCallbackHandle::dead_register(label).dead_summary()
}

