use helper::{format_label, normalize_value, unused_helper as dead_imported_helper};

pub struct LiveModel {
    label: String,
}

impl LiveModel {
    pub fn new(raw: &str) -> Self {
        Self {
            label: normalize_value(raw),
        }
    }

    pub fn render(self) -> String {
        format_label(&self.label)
    }

    pub fn dead_model_method(self) -> String {
        dead_imported_helper(&self.label)
    }
}

pub fn build_model(raw: &str) -> LiveModel {
    LiveModel::new(raw)
}

pub fn model_prefix() -> &'static str {
    "live"
}

pub fn dead_live_model(raw: &str) -> String {
    LiveModel::new(raw).dead_model_method()
}
