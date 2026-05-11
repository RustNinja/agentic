pub struct LiveRecord {
    pub value: u32,
    pub dead_note: Option<String>,
}

pub fn dead_live(seed: u32) -> String {
    LiveRecord {
        value: seed,
        dead_note: Some(format!("dead-{seed}")),
    }
    .dead_note
    .unwrap_or_default()
}

