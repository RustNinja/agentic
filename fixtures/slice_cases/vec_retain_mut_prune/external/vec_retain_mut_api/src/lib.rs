mod live;

pub use live::selected_vec_retain_mut_report;

pub fn dead_vec_retain_mut_report(raw: &str) -> String {
    format!("dead-vec-retain-mut-report:{raw}")
}
