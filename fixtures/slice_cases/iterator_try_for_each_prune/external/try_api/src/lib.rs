mod dead;
mod live;

pub use live::selected_try_for_each_report;

pub fn dead_try_for_each_report(raw: &str) -> String {
    dead::dead_try_for_each_report(raw)
}
