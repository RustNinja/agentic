pub fn selected_summary(seed: u32) -> String {
    let snapshot = audit_record::snapshot(seed);
    let record = snapshot.records.get("live");
    if let Some(record) = record {
        let label = record.label.as_deref().unwrap_or("missing");
        let value = record.value.unwrap_or_default();
        format!("{label}:{value}")
    } else {
        "missing".to_string()
    }
}

pub fn dead_summary(seed: u32) -> String {
    let snapshot = audit_record::dead_snapshot(seed);
    snapshot
        .records
        .get("dead")
        .and_then(|record| record.dead_note.clone())
        .or(snapshot.unused_note)
        .unwrap_or_default()
}
