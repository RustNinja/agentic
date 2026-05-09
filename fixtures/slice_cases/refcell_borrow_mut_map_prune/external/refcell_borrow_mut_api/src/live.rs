pub fn selected_refcell_borrow_mut_map_report(raw: &str) -> String {
    refcell_borrow_mut_model::selected_refcell_borrow_mut_map(raw)
}

pub fn dead_live_refcell_borrow_mut_map_report(raw: &str) -> String {
    format!("dead-live-refcell-borrow-mut-map:{raw}")
}
