mod live;

pub use live::selected_vec_swap_remove_report;

pub fn dead_vec_swap_remove_report(raw: &str) -> String {
    format!("dead-vec-swap-remove-report:{raw}")
}
