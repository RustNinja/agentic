pub enum WireEvent {
    Named(String),
    Count(usize),
}

impl WireEvent {
    pub fn render(self) -> String {
        match self {
            Self::Named(value) => format!("named:{value}"),
            Self::Count(value) => format!("count:{value}"),
        }
    }

    pub fn dead_method(self) -> String {
        format!("dead-enum:{}", self.render())
    }
}

pub fn selected_enum(raw: &str) -> String {
    [raw.trim().to_string()]
        .into_iter()
        .map(WireEvent::Named)
        .map(WireEvent::render)
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_enum(raw: &str) -> String {
    WireEvent::Count(raw.len()).dead_method()
}
