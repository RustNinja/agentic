mod live;

pub use live::selected_collect_option_vec_report;

pub fn dead_collect_option_vec_report(raw: &str) -> String {
    format!("dead-collect-option-vec-report:{raw}")
}
