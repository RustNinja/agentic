pub fn selected_slice_partition_point_report(raw: &str) -> String {
    slice_partition_point_model::selected_slice_partition_point(raw)
}

pub fn dead_live_slice_partition_point_report(raw: &str) -> String {
    format!("dead-slice-partition-point-live-report:{raw}")
}
