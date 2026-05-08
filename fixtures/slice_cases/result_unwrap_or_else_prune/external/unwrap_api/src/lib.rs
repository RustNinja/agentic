mod dead;
mod live;

pub use live::selected_unwrap_report;

pub fn dead_unwrap_report(raw: &str) -> String {
    dead::dead_unwrap_report(raw)
}
