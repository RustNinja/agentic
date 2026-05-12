pub fn selected_summary(seed: u32) -> String {
    let left = left_record::snapshot(seed);
    let right = right_record::snapshot(seed);

    let labels = left
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| format!("{index}:{}", record.label))
        .collect::<Vec<_>>()
        .join(",");
    let total = left
        .records
        .iter()
        .zip(right.entries.iter())
        .map(|(left, right)| left.value + right.weight)
        .sum::<u32>();
    let codes = right
        .entries
        .iter()
        .enumerate()
        .map(|(_, entry)| entry.code.as_str())
        .collect::<Vec<_>>()
        .join(",");

    format!("{labels}:{codes}:{total}")
}

pub fn dead_summary(seed: u32) -> String {
    let left = left_record::dead_snapshot(seed);
    let right = right_record::dead_snapshot(seed);
    left.unused_note
        .into_iter()
        .chain(right.unused_note)
        .chain(left.records.into_iter().filter_map(|record| record.dead_note))
        .chain(right.entries.into_iter().filter_map(|entry| entry.dead_note))
        .collect::<Vec<_>>()
        .join(",")
}
