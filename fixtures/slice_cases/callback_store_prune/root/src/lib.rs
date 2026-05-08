use opensourced::opensourced;

#[opensourced]
pub fn selected_callback_store_report(raw: &str) -> String {
    callback_store_api::selected_callback_store_report(raw)
}

pub fn dead_callback_store_report(raw: &str) -> String {
    callback_store_api::dead_callback_store_report(raw)
}
