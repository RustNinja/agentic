use opensourced::opensourced;

#[opensourced]
pub fn selected_borrow_trait_map_report(raw: &str) -> String {
    borrow_trait_api::selected_borrow_trait_map_report(raw)
}

pub fn dead_borrow_trait_map_report(raw: &str) -> String {
    borrow_trait_api::dead_borrow_trait_map_report(raw)
}
