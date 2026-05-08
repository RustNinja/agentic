macro_rules! render_record {
    ($value:expr) => {{
        let normalized = $value.normalize();
        normalized.render()
    }};
}

pub(crate) use render_record;

pub struct MacroRecord {
    raw: String,
}

impl MacroRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.trim().to_string(),
        }
    }

    pub fn normalize(&self) -> NormalizedRecord {
        NormalizedRecord {
            value: self.raw.to_uppercase(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-macro:{}", self.raw)
    }
}

pub struct NormalizedRecord {
    value: String,
}

impl NormalizedRecord {
    pub fn render(&self) -> String {
        format!("macro:{}", self.value)
    }
}

pub fn selected_macro(raw: &str) -> String {
    let record = MacroRecord::new(raw);
    render_record!(record)
}

pub fn dead_live_macro(raw: &str) -> String {
    MacroRecord::new(raw).dead_method()
}
