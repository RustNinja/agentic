mod live;

pub use live::selected_vec_last_report;

pub fn dead_vec_last_report(raw: &str) -> String {
    format!("dead-vec-last-report:{raw}")
}
