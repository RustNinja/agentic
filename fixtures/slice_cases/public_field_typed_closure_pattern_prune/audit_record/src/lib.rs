pub struct AuditRecord {
    pub value: u32,
    pub dead_note: Option<String>,
}

pub fn placeholder(_seed: u32) -> AuditRecord {
    AuditRecord {
        value: 0,
        dead_note: None,
    }
}

pub fn dead_record(seed: u32) -> AuditRecord {
    AuditRecord {
        value: seed,
        dead_note: Some(format!("dead-{seed}")),
    }
}

