mod live;

pub use live::selected_collect_vec_tuple_report;

pub fn dead_collect_vec_tuple_report(raw: &str) -> String {
    format!("dead-collect-vec-tuple-report:{raw}")
}
