mod dead;
mod live;

pub use live::selected_lazy_report;

pub fn dead_lazy_report(raw: &str) -> String {
    dead::dead_lazy_report(raw)
}
