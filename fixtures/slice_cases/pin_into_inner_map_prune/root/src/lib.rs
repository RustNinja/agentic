use opensourced::opensourced;

#[opensourced]
pub fn selected_pin_into_inner_map_report(raw: &str) -> String {
    pin_into_inner_map_api::selected_pin_into_inner_map_report(raw)
}

pub fn dead_pin_into_inner_map_report(raw: &str) -> String {
    pin_into_inner_map_api::dead_pin_into_inner_map_report(raw)
}
