mod live;

pub use live::selected_vec_pop_report;

pub fn dead_vec_pop_report(raw: &str) -> String {
    format!("dead-vec-pop-report:{raw}")
}
