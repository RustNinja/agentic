mod live;

pub use live::selected_hashset_take_report;

pub fn dead_hashset_take_report(raw: &str) -> String {
    format!("dead-hashset-take-report:{raw}")
}
