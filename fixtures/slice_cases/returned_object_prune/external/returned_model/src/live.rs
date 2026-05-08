pub struct ReturnedSubscription {
    label: String,
}

#[cfg_attr(feature = "ffi", uniffi::export)]
impl ReturnedSubscription {
    pub fn next_event(&self) -> ReturnedEvent {
        ReturnedEvent {
            label: self.label.clone(),
        }
    }

    pub fn close(self) -> String {
        format!("closed:{}", self.label)
    }
}

impl ReturnedSubscription {
    pub fn dead_method(&self) -> String {
        format!("dead-returned:{}", self.label)
    }
}

pub struct ReturnedEvent {
    label: String,
}

impl ReturnedEvent {
    pub fn render(&self) -> String {
        format!("event:{}", self.label)
    }

    pub fn dead_event_method(&self) -> String {
        format!("dead-event:{}", self.label)
    }
}

pub fn open_subscription(raw: &str) -> ReturnedSubscription {
    ReturnedSubscription {
        label: raw.trim().to_string(),
    }
}

pub fn dead_live_returned(raw: &str) -> String {
    open_subscription(raw).dead_method()
}
