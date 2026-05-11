pub struct LiveRecord {
    pub value: u32,
    pub dead_note: Option<String>,
}

pub struct AuditRecord {
    pub value: u32,
    pub label: String,
}

impl AuditRecord {
    pub fn label_only(_seed: u32) -> Self {
        Self {
            value: 0,
            label: "audit".to_string(),
        }
    }

    pub fn dead_value(seed: u32) -> Self {
        Self {
            value: seed,
            label: "dead".to_string(),
        }
    }
}

pub fn selected_summary(seed: u32) -> String {
    let live = LiveRecord {
        value: seed,
        dead_note: None,
    };
    let audit = AuditRecord::label_only(seed);
    format!("{}:{}", live.value, audit.label)
}

pub fn dead_summary(seed: u32) -> String {
    let audit = AuditRecord::dead_value(seed);
    audit.value.to_string()
}

