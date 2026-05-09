pub fn selected_slice_splitn_report(raw: &str) -> String {
    slice_splitn_model::selected_slice_splitn(raw)
}

pub fn dead_live_slice_splitn_report(raw: &str) -> String {
    format!("dead-slice-splitn-live-report:{raw}")
}
