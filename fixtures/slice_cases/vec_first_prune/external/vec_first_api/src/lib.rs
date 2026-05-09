mod live;

pub use live::selected_vec_first_report;

pub fn dead_vec_first_report(raw: &str) -> String {
    format!("dead-vec-first-report:{raw}")
}
