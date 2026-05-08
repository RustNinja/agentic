mod dead;
mod live;

pub use live::selected_any_report;

pub fn dead_any_report(raw: &str) -> String {
    dead::dead_any_report(raw)
}
