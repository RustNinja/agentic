pub struct OptionZipLeft {
    value: String,
}

impl OptionZipLeft {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_left(&self) -> String {
        format!("left:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-zip:{}", self.value)
    }
}

pub struct OptionZipRight {
    value: String,
}

impl OptionZipRight {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render_right(&self) -> String {
        format!("right:{}", self.value)
    }
}

fn option_zip_left(raw: &str) -> Option<OptionZipLeft> {
    (!raw.trim().is_empty()).then(|| OptionZipLeft::new(raw))
}

fn option_zip_right(raw: &str) -> Option<OptionZipRight> {
    Some(OptionZipRight::new(raw))
}

pub fn selected_option_zip(raw: &str) -> String {
    option_zip_left(raw)
        .zip(option_zip_right(raw))
        .map(|(left, right)| format!("{}:{}", left.render_left(), right.render_right()))
        .unwrap_or_else(|| "option-zip:missing".to_string())
}

pub fn dead_live_option_zip(raw: &str) -> String {
    OptionZipLeft::new(raw).dead_method()
}
