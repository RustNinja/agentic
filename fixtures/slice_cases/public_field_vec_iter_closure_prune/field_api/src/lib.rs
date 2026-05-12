pub fn selected_summary(seed: u32) -> String {
    let snapshot = audit_record::snapshot(seed);
    snapshot
        .records
        .iter()
        .map(|record| {
            let label = record.label.as_deref().unwrap_or("missing");
            let value = record.value.unwrap_or_default();
            format!("{label}:{value}")
        })
        .next()
        .unwrap_or_default()
}

pub fn dead_summary(seed: u32) -> String {
    let snapshot = audit_record::dead_snapshot(seed);
    snapshot
        .records
        .iter()
        .filter_map(|record| record.dead_note.clone())
        .chain(snapshot.unused_note)
        .next()
        .unwrap_or_else(|| format!("dead-{seed}"))
}

