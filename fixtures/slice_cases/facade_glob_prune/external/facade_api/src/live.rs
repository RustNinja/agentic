use facade_support::facade::{build_live, LiveRecord};

pub fn selected_facade_report(raw: &str) -> String {
    let record: LiveRecord = build_live(raw);
    record.render()
}

pub fn dead_live_facade_report(raw: &str) -> String {
    format!("dead-live:{raw}")
}
