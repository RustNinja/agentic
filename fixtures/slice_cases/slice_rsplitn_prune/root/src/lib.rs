use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_rsplitn_report(raw: &str) -> String {
    slice_rsplitn_api::selected_slice_rsplitn_report(raw)
}

pub fn dead_slice_rsplitn_report(raw: &str) -> String {
    slice_rsplitn_api::dead_slice_rsplitn_report(raw)
}
