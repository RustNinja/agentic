use opensourced::opensourced;

#[opensourced]
pub fn selected_vec_as_slice_chunks_report(raw: &str) -> String {
    vec_as_slice_chunks_api::selected_vec_as_slice_chunks_report(raw)
}

pub fn dead_vec_as_slice_chunks_report(raw: &str) -> String {
    vec_as_slice_chunks_api::dead_vec_as_slice_chunks_report(raw)
}
