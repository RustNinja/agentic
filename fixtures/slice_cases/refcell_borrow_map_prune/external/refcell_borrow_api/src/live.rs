pub fn selected_refcell_borrow_map_report(raw: &str) -> String {
    refcell_borrow_model::selected_refcell_borrow_map(raw)
}

pub fn dead_live_refcell_borrow_map_report(raw: &str) -> String {
    format!("dead-live-refcell-borrow-map:{raw}")
}
