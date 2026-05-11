pub fn selected_sum(seed: u32) -> u32 {
    let records = [audit_record::placeholder(seed)];
    records
        .into_iter()
        .map(
            |audit_record::AuditRecord { value, .. }: audit_record::AuditRecord| value,
        )
        .sum()
}

pub fn dead_sum(seed: u32) -> String {
    let records = [audit_record::dead_record(seed)];
    records
        .into_iter()
        .map(|audit_record::AuditRecord { dead_note, .. }| dead_note.unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",")
}

