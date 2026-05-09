mod live;

pub use live::selected_fuse_last_report;

pub fn dead_fuse_last_report(raw: &str) -> String {
    format!("dead-fuse-last-report:{raw}")
}
