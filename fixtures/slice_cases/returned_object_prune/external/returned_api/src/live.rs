pub use returned_model::ReturnedSubscription;

pub fn selected_returned_subscription(raw: &str) -> ReturnedSubscription {
    returned_model::open_subscription(raw)
}

pub fn dead_live_returned_report(raw: &str) -> String {
    format!("dead-live-returned:{raw}")
}
