mod live;

pub use live::selected_slice_windows_report;

pub fn dead_slice_windows_report(raw: &str) -> String {
    format!("dead-slice-windows-report:{raw}")
}
