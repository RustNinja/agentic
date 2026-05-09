mod live;

pub use live::selected_hashset_iter_find_report;

pub fn dead_hashset_iter_find_report(raw: &str) -> String {
    format!("dead-hashset-iter-find-report:{raw}")
}
