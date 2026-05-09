use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_rchunks_report(raw: &str) -> String {
    slice_rchunks_api::selected_slice_rchunks_report(raw)
}

pub fn dead_slice_rchunks_report(raw: &str) -> String {
    slice_rchunks_api::dead_slice_rchunks_report(raw)
}
