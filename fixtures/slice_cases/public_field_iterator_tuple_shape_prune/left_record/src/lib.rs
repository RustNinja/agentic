pub struct LeftSnapshot {
    pub records: Vec<LeftRecord>,
    pub unused_note: Option<String>,
}

pub struct LeftRecord {
    pub label: String,
    pub value: u32,
    pub dead_note: Option<String>,
}

pub fn snapshot(seed: u32) -> LeftSnapshot {
    let mut records = Vec::new();
    records.push(LeftRecord {
        label: "left".to_string(),
        value: seed,
        dead_note: None,
    });
    LeftSnapshot {
        records,
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> LeftSnapshot {
    let mut snapshot = snapshot(seed);
    snapshot.unused_note = Some("unused".to_string());
    if let Some(first) = snapshot.records.get_mut(0) {
        first.dead_note = Some("unused".to_string());
    }
    snapshot
}
