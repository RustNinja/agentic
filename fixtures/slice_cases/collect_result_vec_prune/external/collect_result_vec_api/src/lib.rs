mod live;

pub use live::selected_collect_result_vec_report;

pub fn dead_collect_result_vec_report(raw: &str) -> String {
    format!("dead-collect-result-vec-report:{raw}")
}
