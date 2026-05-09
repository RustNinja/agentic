mod live;

pub use live::selected_vec_get_report;

pub fn dead_vec_get_report(raw: &str) -> String {
    format!("dead-vec-get-report:{raw}")
}
