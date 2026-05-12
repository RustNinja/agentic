pub struct RightSnapshot {
    pub entries: Vec<RightRecord>,
    pub unused_note: Option<String>,
}

pub struct RightRecord {
    pub code: String,
    pub weight: u32,
    pub loop_code: String,
    pub loop_weight: u32,
    pub dead_note: Option<String>,
}

pub fn snapshot(seed: u32) -> RightSnapshot {
    let mut entries = Vec::new();
    entries.push(RightRecord {
        code: "right".to_string(),
        weight: seed + 1,
        loop_code: "right-loop".to_string(),
        loop_weight: seed + 11,
        dead_note: None,
    });
    RightSnapshot {
        entries,
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> RightSnapshot {
    let mut snapshot = snapshot(seed);
    snapshot.unused_note = Some("unused".to_string());
    if let Some(first) = snapshot.entries.get_mut(0) {
        first.dead_note = Some("unused".to_string());
    }
    snapshot
}
