use opensourced::opensourced;
use session_api::{SessionHandle, SessionRequest, SessionStatusDto};

#[opensourced]
pub fn connect_session_status(label: &str) -> SessionStatusDto {
    let request = SessionRequest::new(label);
    let handle = SessionHandle::connect(request);
    handle.status()
}

pub fn dead_session_status(label: &str) -> String {
    SessionHandle::dead_connect(label).dead_summary()
}

