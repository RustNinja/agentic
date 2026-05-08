pub type StreamResult = Result<EventEnvelope, StreamError>;

pub struct EventDto {
    label: String,
}

impl EventDto {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

pub struct EventEnvelope {
    events: Vec<EventDto>,
}

impl EventEnvelope {
    pub fn render(self) -> String {
        self.events
            .iter()
            .map(EventDto::label)
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub struct StreamError {
    message: String,
}

impl StreamError {
    pub fn render(self) -> String {
        format!("stream-error:{}", self.message)
    }
}

pub fn event_iter(raw: &str) -> impl Iterator<Item = EventDto> {
    [EventDto::new(raw), EventDto::new("tail")].into_iter()
}

pub fn selected_stream(raw: &str) -> StreamResult {
    if raw.trim().is_empty() {
        Err(StreamError {
            message: "empty".to_string(),
        })
    } else {
        Ok(EventEnvelope {
            events: event_iter(raw).collect(),
        })
    }
}

pub fn dead_live_stream(raw: &str) -> usize {
    event_iter(raw).count() + 99
}
