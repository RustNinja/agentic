mod live;

pub use live::selected_slice_rsplitn_report;

pub fn dead_slice_rsplitn_report(raw: &str) -> String {
    format!("dead-slice-rsplitn-report:{raw}")
}
