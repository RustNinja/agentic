use opensourced::opensourced;

#[opensourced]
pub fn selected_returned_subscription(raw: &str) -> returned_api::ReturnedSubscription {
    returned_api::selected_returned_subscription(raw)
}

pub fn dead_returned_report(raw: &str) -> String {
    returned_api::dead_returned_report(raw)
}
