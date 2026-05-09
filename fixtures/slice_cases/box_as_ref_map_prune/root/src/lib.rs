use opensourced::opensourced;

#[opensourced]
pub fn selected_box_as_ref_map_report(raw: &str) -> String {
    box_as_ref_api::selected_box_as_ref_map_report(raw)
}

pub fn dead_box_as_ref_map_report(raw: &str) -> String {
    box_as_ref_api::dead_box_as_ref_map_report(raw)
}
