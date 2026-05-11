pub fn selected(seed: u32) -> String {
    let live = live_record::LiveRecord {
        value: seed,
        dead_note: None,
    };
    format!("{}:{}", live.value, audit_record::AuditRecord::label_only(seed))
}

pub fn dead(seed: u32) -> String {
    format!(
        "{}:{}",
        live_record::dead_live(seed),
        audit_record::dead_audit(seed)
    )
}

