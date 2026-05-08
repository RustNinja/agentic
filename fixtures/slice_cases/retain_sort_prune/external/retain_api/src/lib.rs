mod dead;
mod live;

pub use live::selected_retain_report;

pub fn dead_retain_report(raw: &str) -> String {
    dead::dead_retain_report(raw)
}
