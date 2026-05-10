use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_rchunks_mut_for_pattern_report(raw: &str) -> String {
    slice_rchunks_mut_for_pattern_api::selected_slice_rchunks_mut_for_pattern_report(raw)
}

pub fn dead_slice_rchunks_mut_for_pattern_report(raw: &str) -> String {
    slice_rchunks_mut_for_pattern_api::dead_slice_rchunks_mut_for_pattern_report(raw)
}
