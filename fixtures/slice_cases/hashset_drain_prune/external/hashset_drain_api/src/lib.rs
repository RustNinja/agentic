mod live;

pub use live::selected_hashset_drain_report;

pub fn dead_hashset_drain_report(raw: &str) -> String {
    format!("dead-hashset-drain-report:{raw}")
}
