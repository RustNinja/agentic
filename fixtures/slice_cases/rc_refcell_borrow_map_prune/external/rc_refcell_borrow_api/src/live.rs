pub fn selected_rc_refcell_borrow_map_report(raw: &str) -> String {
    rc_refcell_borrow_model::selected_rc_refcell_borrow_map(raw)
}

pub fn dead_live_rc_refcell_borrow_map_report(raw: &str) -> String {
    format!("dead-live-rc-refcell-borrow-map:{raw}")
}
