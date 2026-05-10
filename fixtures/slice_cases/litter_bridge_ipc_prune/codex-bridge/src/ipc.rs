use bridge_core::{dispatch_method, BridgeSession};
use bridge_protocol::{BridgeError, Method, ResponseFrame, WireFrame};
use opensourced::opensourced;

#[opensourced]
pub fn handle_frame(raw: &str) -> Result<ResponseFrame, BridgeError> {
    let frame = WireFrame::decode(raw)?;
    let method = Method::from_wire(frame.method())?;
    let session = BridgeSession::new(frame.session_id());
    let payload = dispatch_method(&session, method, frame.payload())?;
    Ok(ResponseFrame::ok(session.id(), payload))
}

pub fn dead_handle_frame(raw: &str) -> Result<ResponseFrame, BridgeError> {
    let frame = WireFrame::decode(raw)?;
    Err(BridgeError::dead(frame.payload()))
}

