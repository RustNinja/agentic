pub struct RawItem {
    value: String,
}

impl RawItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn clean(self) -> Result<CleanItem, CleanError> {
        if self.value.is_empty() {
            Err(CleanError::Empty)
        } else {
            Ok(CleanItem { value: self.value })
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-raw:{}", self.value)
    }
}

pub struct CleanItem {
    value: String,
}

impl CleanItem {
    pub fn render(&self) -> String {
        format!("clean:{}", self.value)
    }
}

pub enum CleanError {
    Empty,
}

impl CleanError {
    pub fn message(&self) -> String {
        "empty".to_string()
    }
}

pub fn selected_closure(raw: &str) -> String {
    Some(RawItem::new(raw))
        .map(|item| item.clean())
        .transpose()
        .map(|item| item.map(|clean| clean.render()).unwrap_or_default())
        .unwrap_or_else(|error| error.message())
}

pub fn dead_live_closure(raw: &str) -> String {
    RawItem::new(raw).dead_method()
}
