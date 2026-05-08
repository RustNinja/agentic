use std::fmt;

pub struct DisplayRecord {
    label: String,
}

impl DisplayRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-display:{}", self.label)
    }
}

impl fmt::Display for DisplayRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "display:{}", self.label)
    }
}

pub fn selected_display(raw: &str) -> String {
    let record = DisplayRecord::new(raw);
    format!("record:{record}")
}

pub fn dead_live_display(raw: &str) -> String {
    DisplayRecord::new(raw).dead_method()
}
