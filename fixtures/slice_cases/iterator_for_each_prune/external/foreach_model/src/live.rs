pub struct ForEachEvent {
    value: String,
}

impl ForEachEvent {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("each:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-each:{}", self.value)
    }
}

fn build_events(raw: &str) -> Vec<ForEachEvent> {
    raw.split(',').map(ForEachEvent::new).collect()
}

pub fn selected_for_each(raw: &str) -> String {
    let mut rendered = Vec::new();
    build_events(raw)
        .iter()
        .for_each(|event| rendered.push(event.render()));
    rendered.join("|")
}

pub fn dead_live_for_each(raw: &str) -> String {
    ForEachEvent::new(raw).dead_method()
}
