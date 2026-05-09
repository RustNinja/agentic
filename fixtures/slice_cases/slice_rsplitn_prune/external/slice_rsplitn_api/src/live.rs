pub fn selected_slice_rsplitn_report(raw: &str) -> String {
    slice_rsplitn_model::selected_slice_rsplitn(raw)
}

pub fn dead_live_slice_rsplitn_report(raw: &str) -> String {
    format!("dead-slice-rsplitn-live-report:{raw}")
}
