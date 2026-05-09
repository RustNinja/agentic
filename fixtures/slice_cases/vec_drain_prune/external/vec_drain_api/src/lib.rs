mod live;

pub use live::selected_vec_drain_report;

pub fn dead_vec_drain_report(raw: &str) -> String {
    format!("dead-vec-drain-report:{raw}")
}
