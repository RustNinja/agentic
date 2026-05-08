mod dead;
mod live;

pub use live::selected_fold_report;

pub fn dead_fold_report(raw: &str) -> String {
    dead::dead_fold_report(raw)
}
