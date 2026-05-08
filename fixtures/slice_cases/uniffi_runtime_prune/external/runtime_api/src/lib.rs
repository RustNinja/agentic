mod dead;
mod live;

pub use live::selected_runtime_report;

pub fn dead_runtime_report(raw: &str) -> String {
    dead::dead_runtime_report(raw)
}
