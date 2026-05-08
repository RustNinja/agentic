mod dead;
mod live;

pub use live::selected_filter_report;

pub fn dead_filter_report(raw: &str) -> String {
    dead::dead_filter_report(raw)
}
