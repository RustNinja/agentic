mod dead;
mod live;

pub use live::selected_for_each_report;

pub fn dead_for_each_report(raw: &str) -> String {
    dead::dead_for_each_report(raw)
}
