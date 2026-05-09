mod live;

pub use live::selected_hashset_into_iter_report;

pub fn dead_hashset_into_iter_report(raw: &str) -> String {
    format!("dead-hashset-into-iter-report:{raw}")
}
