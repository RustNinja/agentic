use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_splitn_report(raw: &str) -> String {
    slice_splitn_api::selected_slice_splitn_report(raw)
}

pub fn dead_slice_splitn_report(raw: &str) -> String {
    slice_splitn_api::dead_slice_splitn_report(raw)
}
