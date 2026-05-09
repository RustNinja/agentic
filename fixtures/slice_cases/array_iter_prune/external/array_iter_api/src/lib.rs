mod live;

pub use live::selected_array_iter_report;

pub fn dead_array_iter_report(raw: &str) -> String {
    format!("dead-array-iter-report:{raw}")
}
