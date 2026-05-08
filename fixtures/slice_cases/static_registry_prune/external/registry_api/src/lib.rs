mod dead;
mod live;

pub use live::selected_registry_report;

pub fn dead_registry_report(raw: &str) -> String {
    dead::dead_registry_report(raw)
}
