use opensourced::opensourced;

#[opensourced]
pub fn selected_refcell_borrow_map_report(raw: &str) -> String {
    refcell_borrow_api::selected_refcell_borrow_map_report(raw)
}

pub fn dead_refcell_borrow_map_report(raw: &str) -> String {
    refcell_borrow_api::dead_refcell_borrow_map_report(raw)
}
