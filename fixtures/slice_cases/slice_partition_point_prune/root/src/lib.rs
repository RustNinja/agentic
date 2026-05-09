use opensourced::opensourced;

#[opensourced]
pub fn selected_slice_partition_point_report(raw: &str) -> String {
    slice_partition_point_api::selected_slice_partition_point_report(raw)
}

pub fn dead_slice_partition_point_report(raw: &str) -> String {
    slice_partition_point_api::dead_slice_partition_point_report(raw)
}
