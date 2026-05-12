pub struct AuditSnapshot {
    pub records: Vec<AuditRecord>,
    pub unused_note: Option<String>,
}

pub struct AuditRecord {
    pub value: Option<u32>,
    pub label: Option<String>,
    pub dead_note: Option<String>,
}

pub fn snapshot(_seed: u32) -> AuditSnapshot {
    let mut records = Vec::new();
    records.push(AuditRecord {
        value: None,
        label: None,
        dead_note: None,
    });
    AuditSnapshot {
        records,
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> AuditSnapshot {
    AuditSnapshot {
        records: vec![AuditRecord {
            value: Some(seed),
            label: Some(format!("dead-{seed}")),
            dead_note: Some(format!("unused-{seed}")),
        }],
        unused_note: Some(format!("unused-{seed}")),
    }
}
