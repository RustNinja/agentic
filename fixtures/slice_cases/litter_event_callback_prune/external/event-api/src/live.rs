use std::sync::Arc;

use event_core::{EventBus, EventCallback, EventEnvelope, EventKind};
use event_protocol::{WireEvent, WireEventKind};

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EventRegistration {
    label: String,
}

impl EventRegistration {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.trim().to_string(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg_attr(feature = "ffi", derive(uniffi::Object))]
pub struct EventCallbackHandle {
    bus: EventBus,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
impl EventCallbackHandle {
    pub fn register(registration: EventRegistration) -> Self {
        let bus = EventBus::new(registration.label());
        bus.set_callback(Arc::new(DefaultCallback));
        Self { bus }
    }

    pub fn emit_preview(&self, raw: &str) -> EventSnapshotDto {
        EventSnapshotDto::from_wire(self.bus.emit(raw))
    }

    pub fn dead_exported_preview(&self) -> String {
        self.bus.dead_debug()
    }
}

impl EventCallbackHandle {
    pub fn dead_register(label: &str) -> DeadEventApi {
        DeadEventApi {
            label: label.to_string(),
        }
    }
}

struct DefaultCallback;

impl EventCallback for DefaultCallback {
    fn on_event(&self, event: EventEnvelope) -> WireEvent {
        match event.kind() {
            EventKind::Connected => WireEvent::connected(event.label(), event.payload()),
            EventKind::Message => WireEvent::message(event.label(), event.payload()),
            EventKind::Closed => WireEvent::closed(event.label(), event.payload()),
        }
    }
}

#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct EventSnapshotDto {
    pub label: String,
    pub body: String,
    pub connected: bool,
}

impl EventSnapshotDto {
    pub fn from_wire(event: WireEvent) -> Self {
        Self {
            label: event.label().to_string(),
            body: event.body().to_string(),
            connected: matches!(event.kind(), WireEventKind::Connected),
        }
    }

    pub fn dead_render(&self) -> String {
        format!("dead-snapshot:{}", self.label)
    }
}

pub struct DeadEventApi {
    label: String,
}

impl DeadEventApi {
    pub fn dead_summary(self) -> String {
        format!("dead-event-api:{}", self.label)
    }
}

pub fn dead_live_event_api(label: &str) -> String {
    DeadEventApi {
        label: label.to_string(),
    }
    .dead_summary()
}

