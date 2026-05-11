pub fn selected_score(seed: u32) -> u32 {
    let record = audit_record::placeholder(seed);
    read_value(record)
}

fn read_value(record: audit_record::AuditRecord) -> u32 {
    record.value
}

pub fn dead_score(seed: u32) -> String {
    let record = audit_record::dead_record(seed);
    dead_note(record)
}

fn dead_note(record: audit_record::AuditRecord) -> String {
    record.dead_note.unwrap_or_default()
}

