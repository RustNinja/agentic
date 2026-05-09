pub fn selected_slice_windows_report(raw: &str) -> String {
    slice_windows_model::selected_slice_windows(raw)
}

pub fn dead_live_slice_windows_report(raw: &str) -> String {
    format!("dead-slice-windows-live-report:{raw}")
}
