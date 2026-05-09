use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_rsplit_report(raw: &str) -> String {
    slice_rsplit_api::selected_slice_rsplit_report(raw)
}

pub fn dead_slice_rsplit_report(raw: &str) -> String {
    slice_rsplit_api::dead_slice_rsplit_report(raw)
}
