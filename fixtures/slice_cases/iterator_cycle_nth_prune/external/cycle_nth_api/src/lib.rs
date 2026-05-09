mod live;

pub use live::selected_cycle_nth_report;

pub fn dead_cycle_nth_report(raw: &str) -> String {
    format!("dead-cycle-nth-report:{raw}")
}
