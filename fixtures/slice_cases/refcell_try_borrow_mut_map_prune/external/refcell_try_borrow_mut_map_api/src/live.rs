pub fn selected_refcell_try_borrow_mut_map_report(raw: &str) -> String {
    refcell_try_borrow_mut_map_model::selected_refcell_try_borrow_mut_map(raw)
}

pub fn dead_live_refcell_try_borrow_mut_map_report(raw: &str) -> String {
    format!("dead-live-refcell-try-borrow-mut-map-report:{raw}")
}
