use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_box_iter_as_ref_map_report(raw: &str) -> String {
    vec_box_iter_as_ref_api::selected_vec_box_iter_as_ref_map_report(raw)
}

pub fn dead_vec_box_iter_as_ref_map_report(raw: &str) -> String {
    vec_box_iter_as_ref_api::dead_vec_box_iter_as_ref_map_report(raw)
}
