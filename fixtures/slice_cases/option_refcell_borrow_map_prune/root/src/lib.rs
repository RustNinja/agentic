use opensourced::opensourced;

#[opensourced]
pub fn selected_option_refcell_borrow_map_report(raw: &str) -> String {
    option_refcell_borrow_api::selected_option_refcell_borrow_map_report(raw)
}

pub fn dead_option_refcell_borrow_map_report(raw: &str) -> String {
    option_refcell_borrow_api::dead_option_refcell_borrow_map_report(raw)
}
