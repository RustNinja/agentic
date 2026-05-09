use opensourced::opensourced;

#[opensourced]
pub fn selected_rc_refcell_borrow_map_report(raw: &str) -> String {
    rc_refcell_borrow_api::selected_rc_refcell_borrow_map_report(raw)
}

pub fn dead_rc_refcell_borrow_map_report(raw: &str) -> String {
    rc_refcell_borrow_api::dead_rc_refcell_borrow_map_report(raw)
}
