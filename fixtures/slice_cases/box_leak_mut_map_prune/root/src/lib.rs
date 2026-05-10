use opensourced::opensourced;

#[opensourced]
pub fn selected_box_leak_mut_map_report(raw: &str) -> String {
    box_leak_mut_map_api::selected_box_leak_mut_map_report(raw)
}

pub fn dead_box_leak_mut_map_report(raw: &str) -> String {
    box_leak_mut_map_api::dead_box_leak_mut_map_report(raw)
}
