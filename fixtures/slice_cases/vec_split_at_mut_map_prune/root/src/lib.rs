use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_split_at_mut_map_report(raw: &str) -> String {
    vec_split_at_mut_map_api::selected_vec_split_at_mut_map_report(raw)
}

pub fn dead_vec_split_at_mut_map_report(raw: &str) -> String {
    vec_split_at_mut_map_api::dead_vec_split_at_mut_map_report(raw)
}
