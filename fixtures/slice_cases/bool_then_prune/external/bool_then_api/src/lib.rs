mod live;

pub use live::selected_bool_then_report;

pub fn dead_bool_then_report(raw: &str) -> String {
    format!("dead-bool-then-report:{raw}")
}
