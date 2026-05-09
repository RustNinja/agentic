use opensourced::opensourced;

#[opensourced]
pub fn selected_rc_as_ref_map_report(raw: &str) -> String {
    rc_as_ref_api::selected_rc_as_ref_map_report(raw)
}

pub fn dead_rc_as_ref_map_report(raw: &str) -> String {
    rc_as_ref_api::dead_rc_as_ref_map_report(raw)
}
