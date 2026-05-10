use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_chunks_exact_map_report(raw: &str) -> String {
    vec_chunks_exact_map_api::selected_vec_chunks_exact_map_report(raw)
}

pub fn dead_vec_chunks_exact_map_report(raw: &str) -> String {
    vec_chunks_exact_map_api::dead_vec_chunks_exact_map_report(raw)
}
