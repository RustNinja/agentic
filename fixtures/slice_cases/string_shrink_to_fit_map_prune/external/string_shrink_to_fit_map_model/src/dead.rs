pub struct DeadStringShrinkToFitMapItem {
    value: String,
}

impl DeadStringShrinkToFitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-shrink-to-fit-map:{}", self.value)
    }
}

pub fn dead_string_shrink_to_fit_map(raw: &str) -> String {
    DeadStringShrinkToFitMapItem::new(raw).dead_method()
}
