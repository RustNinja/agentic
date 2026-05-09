mod live;

pub use live::selected_bool_then_some_report;

pub fn dead_bool_then_some_report(raw: &str) -> String {
    format!("dead-bool-then-some-report:{raw}")
}
