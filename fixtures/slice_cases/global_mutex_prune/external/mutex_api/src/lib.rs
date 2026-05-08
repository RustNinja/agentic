mod dead;
mod live;

pub use live::selected_mutex_report;

pub fn dead_mutex_report(raw: &str) -> String {
    dead::dead_mutex_report(raw)
}
