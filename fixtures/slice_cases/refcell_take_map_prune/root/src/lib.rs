use opensourced::opensourced;

#[opensourced]
pub fn selected_refcell_take_map_report(raw: &str) -> String {
    refcell_take_map_api::selected_refcell_take_map_report(raw)
}

pub fn dead_refcell_take_map_report(raw: &str) -> String {
    refcell_take_map_api::dead_refcell_take_map_report(raw)
}
