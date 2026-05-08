use facade_support::facade::{dead_factory, DeadRecord};

pub fn dead_facade_report(raw: &str) -> String {
    let record: DeadRecord = dead_factory(raw);
    record.render_dead()
}
