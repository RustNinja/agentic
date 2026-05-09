mod live;

pub use live::selected_slice_splitn_report;

pub fn dead_slice_splitn_report(raw: &str) -> String {
    format!("dead-slice-splitn-report:{raw}")
}
