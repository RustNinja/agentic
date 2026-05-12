use std::collections::HashMap;

pub struct AuditSnapshot {
    pub records: HashMap<String, AuditRecord>,
    pub unused_note: Option<String>,
}

pub struct AuditRecord {
    pub value: Option<u32>,
    pub label: Option<String>,
    pub dead_note: Option<String>,
}

pub fn snapshot(seed: u32) -> AuditSnapshot {
    let mut records = HashMap::new();
    records.insert(
        "live".to_string(),
        AuditRecord {
            value: None,
            label: None,
            dead_note: None,
        },
    );
    AuditSnapshot {
        records,
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> AuditSnapshot {
    let mut records = HashMap::new();
    records.insert(
        "dead".to_string(),
        AuditRecord {
            value: Some(seed),
            label: Some(format!("dead-{seed}")),
            dead_note: Some(format!("unused-{seed}")),
        },
    );
    AuditSnapshot {
        records,
        unused_note: Some(format!("unused-{seed}")),
    }
}
