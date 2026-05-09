mod live;

pub use live::selected_vec_remove_report;

pub fn dead_vec_remove_report(raw: &str) -> String {
    format!("dead-vec-remove-report:{raw}")
}
