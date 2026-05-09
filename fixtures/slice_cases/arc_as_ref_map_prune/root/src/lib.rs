use opensourced::opensourced;

#[opensourced]
pub fn selected_arc_as_ref_map_report(raw: &str) -> String {
    arc_as_ref_api::selected_arc_as_ref_map_report(raw)
}

pub fn dead_arc_as_ref_map_report(raw: &str) -> String {
    arc_as_ref_api::dead_arc_as_ref_map_report(raw)
}
