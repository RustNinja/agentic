mod live;

pub use live::selected_btreemap_get_mut_report;

pub fn dead_btreemap_get_mut_report(raw: &str) -> String {
    format!("dead-btreemap-get-mut-report:{raw}")
}
