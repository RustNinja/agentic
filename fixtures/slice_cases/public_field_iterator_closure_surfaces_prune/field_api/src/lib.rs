pub fn selected_summary(seed: u32) -> String {
    let mut snapshot = audit_record::snapshot(seed);
    snapshot.records.sort_by(|left, right| left.label.cmp(&right.label));

    let total = snapshot.records.iter().fold(0, |acc, record| {
        acc + record.value.unwrap_or_default()
    });
    let tags = snapshot
        .records
        .iter()
        .flat_map(|record| record.tags.iter())
        .fold(String::new(), |mut acc, tag| {
            acc.push_str(tag);
            acc
        });
    let active = snapshot
        .records
        .iter()
        .partition::<Vec<_>, _>(|record| record.active)
        .0
        .len();

    format!("{total}:{tags}:{active}")
}

pub fn dead_summary(seed: u32) -> String {
    let snapshot = audit_record::dead_snapshot(seed);
    snapshot
        .records
        .iter()
        .filter_map(|record| record.dead_note.as_ref())
        .chain(snapshot.unused_note.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join(",")
}
