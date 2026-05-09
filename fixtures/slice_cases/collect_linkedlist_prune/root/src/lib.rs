use opensourced::opensourced;

#[opensourced]
pub fn selected_collect_linkedlist_report(raw: &str) -> String {
    collect_linkedlist_api::selected_collect_linkedlist_report(raw)
}

pub fn dead_collect_linkedlist_report(raw: &str) -> String {
    collect_linkedlist_api::dead_collect_linkedlist_report(raw)
}
