use opensourced::opensourced;

#[opensourced]
pub fn selected_refcell_try_borrow_mut_map_report(raw: &str) -> String {
    refcell_try_borrow_mut_map_api::selected_refcell_try_borrow_mut_map_report(raw)
}

pub fn dead_refcell_try_borrow_mut_map_report(raw: &str) -> String {
    refcell_try_borrow_mut_map_api::dead_refcell_try_borrow_mut_map_report(raw)
}
