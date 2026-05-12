pub struct AuditSnapshot {
    pub records: Vec<AuditRecord>,
    pub unused_note: Option<String>,
}

pub struct AuditRecord {
    pub value: Option<u32>,
    pub label: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub dead_note: Option<String>,
}

pub fn snapshot(seed: u32) -> AuditSnapshot {
    let mut records = Vec::new();
    records.push(AuditRecord {
        value: Some(seed),
        label: "primary".to_string(),
        tags: {
            let mut tags = Vec::new();
            tags.push("fast".to_string());
            tags
        },
        active: seed > 0,
        dead_note: None,
    });
    AuditSnapshot {
        records,
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> AuditSnapshot {
    let mut snapshot = snapshot(seed);
    snapshot.unused_note = Some("unused".to_string());
    if let Some(first) = snapshot.records.get_mut(0) {
        first.dead_note = Some("unused".to_string());
    }
    snapshot
}
