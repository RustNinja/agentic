use std::sync::{Arc, RwLock};

pub trait EventSink {
    fn on_event(&self, event: EventRecord) -> String;

    fn dead_trait_method(&self) -> String {
        "dead-callback-trait".to_string()
    }
}

pub struct EventRecord {
    value: String,
}

impl EventRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("event:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-event:{}", self.value)
    }
}

pub struct CallbackStore {
    sink: Arc<RwLock<Option<Arc<dyn EventSink + Send + Sync>>>>,
}

impl CallbackStore {
    pub fn new() -> Self {
        Self {
            sink: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_sink(&self, sink: Arc<dyn EventSink + Send + Sync>) {
        *self.sink.write().expect("callback lock should not poison") = Some(sink);
    }

    pub fn emit(&self, raw: &str) -> String {
        let event = EventRecord::new(raw);
        self.sink
            .read()
            .expect("callback lock should not poison")
            .as_ref()
            .map(|sink| sink.on_event(event))
            .unwrap_or_else(|| "missing".to_string())
    }

    pub fn dead_method(&self) -> String {
        "dead-store".to_string()
    }
}

struct DefaultSink;

impl EventSink for DefaultSink {
    fn on_event(&self, event: EventRecord) -> String {
        event.render()
    }
}

pub fn selected_callback_store(raw: &str) -> String {
    let store = CallbackStore::new();
    store.set_sink(Arc::new(DefaultSink));
    store.emit(raw)
}

pub fn dead_live_callback_store(raw: &str) -> String {
    EventRecord::new(raw).dead_method()
}
