#[derive(Clone)]
pub struct LiveWire {
    label: String,
}

impl LiveWire {
    pub fn new(raw: &str) -> Self {
        Self {
            label: normalize_label(raw),
        }
    }

    pub fn render(self) -> String {
        format!("wire:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-wire:{}", self.label)
    }
}

fn normalize_label(raw: &str) -> String {
    raw.trim().to_string()
}

pub fn wire_tag() -> &'static str {
    "wire"
}

pub fn dead_live_wire(raw: &str) -> String {
    LiveWire::new(raw).dead_method()
}
