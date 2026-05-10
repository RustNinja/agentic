use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_dedup_by_map_report(raw: &str) -> String {
    vec_dedup_by_map_api::selected_vec_dedup_by_map_report(raw)
}

pub fn dead_vec_dedup_by_map_report(raw: &str) -> String {
    vec_dedup_by_map_api::dead_vec_dedup_by_map_report(raw)
}
