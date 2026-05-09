use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_chunks_report(raw: &str) -> String {
    slice_chunks_api::selected_slice_chunks_report(raw)
}

pub fn dead_slice_chunks_report(raw: &str) -> String {
    slice_chunks_api::dead_slice_chunks_report(raw)
}
