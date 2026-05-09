use opensourced::opensourced;

#[opensourced]
pub fn selected_pin_box_as_mut_map_report(raw: &str) -> String {
    pin_box_as_mut_api::selected_pin_box_as_mut_map_report(raw)
}

pub fn dead_pin_box_as_mut_map_report(raw: &str) -> String {
    pin_box_as_mut_api::dead_pin_box_as_mut_map_report(raw)
}
