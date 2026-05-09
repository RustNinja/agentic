mod live;

pub use live::selected_slice_partition_point_report;

pub fn dead_slice_partition_point_report(raw: &str) -> String {
    format!("dead-slice-partition-point-report:{raw}")
}
