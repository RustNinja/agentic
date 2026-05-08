pub struct StopParams {
    reason: String,
}

pub struct DeadDispatch {
    value: String,
}

pub fn dead_dispatch(raw: &str) -> String {
    let stop = StopParams {
        reason: raw.to_string(),
    };
    format!(
        "dead-dispatch-model:{}:{}",
        DeadDispatch { value: raw.into() }.value,
        stop.reason
    )
}
