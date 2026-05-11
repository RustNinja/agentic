pub struct AuditRecord {
    pub value: u32,
    pub label: String,
}

impl AuditRecord {
    pub fn label_only(seed: u32) -> String {
        format!("audit-{seed}")
    }

    pub fn dead_value(self) -> u32 {
        self.value
    }

    pub fn dead_label(self) -> String {
        self.label
    }
}

pub fn dead_audit(seed: u32) -> u32 {
    AuditRecord {
        value: seed,
        label: format!("dead-{seed}"),
    }
    .dead_value()
}

