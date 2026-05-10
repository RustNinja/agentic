use opensourced::opensourced;

#[opensourced]
pub fn selected_rc_into_inner_map_report(raw: &str) -> String {
    rc_into_inner_map_api::selected_rc_into_inner_map_report(raw)
}

pub fn dead_rc_into_inner_map_report(raw: &str) -> String {
    rc_into_inner_map_api::dead_rc_into_inner_map_report(raw)
}
