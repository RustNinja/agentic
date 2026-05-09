use opensourced::opensourced;

#[opensourced]
pub fn selected_box_pin_as_ref_map_report(raw: &str) -> String {
    box_pin_as_ref_map_api::selected_box_pin_as_ref_map_report(raw)
}

pub fn dead_box_pin_as_ref_map_report(raw: &str) -> String {
    box_pin_as_ref_map_api::dead_box_pin_as_ref_map_report(raw)
}
