pub struct SourceSnapshot {
    pub records: Vec<SourceRecord>,
    pub unused_note: Option<String>,
}

pub struct SourceRecord {
    pub raw_label: String,
    pub raw_value: u32,
    pub filter_code: String,
    pub children: Vec<SourceChild>,
    pub dead_note: Option<String>,
}

pub struct SourceChild {
    pub child_label: String,
    pub child_weight: u32,
    pub dead_child_note: Option<String>,
}

pub fn snapshot(seed: u32) -> SourceSnapshot {
    let mut records = Vec::new();
    records.push(SourceRecord {
        raw_label: "source".to_string(),
        raw_value: seed + 1,
        filter_code: "filter".to_string(),
        children: {
            let mut children = Vec::new();
            children.push(SourceChild {
                child_label: "child".to_string(),
                child_weight: seed + 2,
                dead_child_note: None,
            });
            children
        },
        dead_note: None,
    });
    SourceSnapshot {
        records,
        unused_note: None,
    }
}

pub fn dead_snapshot(seed: u32) -> SourceSnapshot {
    let mut snapshot = snapshot(seed);
    snapshot.unused_note = Some("unused".to_string());
    if let Some(first) = snapshot.records.get_mut(0) {
        first.dead_note = Some("unused".to_string());
        if let Some(child) = first.children.get_mut(0) {
            child.dead_child_note = Some("unused".to_string());
        }
    }
    snapshot
}
