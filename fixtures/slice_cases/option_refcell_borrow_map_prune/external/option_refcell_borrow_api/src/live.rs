pub fn selected_option_refcell_borrow_map_report(raw: &str) -> String {
    option_refcell_borrow_model::selected_option_refcell_borrow_map(raw)
}

pub fn dead_live_option_refcell_borrow_map_report(raw: &str) -> String {
    format!("dead-live-option-refcell-borrow-map:{raw}")
}
