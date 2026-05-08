pub enum PatternEvent {
    Started { id: u32, label: String },
    Ignored,
}

impl PatternEvent {
    pub fn dead_method(&self) -> String {
        "dead-pattern-event".to_string()
    }
}

pub fn parse_event(raw: &str) -> Option<PatternEvent> {
    Some(PatternEvent::Started {
        id: raw.trim().len() as u32,
        label: raw.trim().to_string(),
    })
}

pub fn selected_pattern(raw: &str) -> String {
    let Some(PatternEvent::Started { id, label }) = parse_event(raw) else {
        return "missing".to_string();
    };
    format!("pattern:{id}:{label}")
}

pub fn dead_live_pattern(raw: &str) -> String {
    parse_event(raw)
        .map(|event| event.dead_method())
        .unwrap_or_else(|| "dead-pattern".to_string())
}
