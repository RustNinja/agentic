use opensourced::opensourced;

#[opensourced]
pub fn selected_arc_try_unwrap_ok_map_report(raw: &str) -> String {
    arc_try_unwrap_ok_map_api::selected_arc_try_unwrap_ok_map_report(raw)
}

pub fn dead_arc_try_unwrap_ok_map_report(raw: &str) -> String {
    arc_try_unwrap_ok_map_api::dead_arc_try_unwrap_ok_map_report(raw)
}
