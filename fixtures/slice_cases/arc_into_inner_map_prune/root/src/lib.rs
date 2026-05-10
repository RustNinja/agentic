use opensourced::opensourced;

#[opensourced]
pub fn selected_arc_into_inner_map_report(raw: &str) -> String {
    arc_into_inner_map_api::selected_arc_into_inner_map_report(raw)
}

pub fn dead_arc_into_inner_map_report(raw: &str) -> String {
    arc_into_inner_map_api::dead_arc_into_inner_map_report(raw)
}
