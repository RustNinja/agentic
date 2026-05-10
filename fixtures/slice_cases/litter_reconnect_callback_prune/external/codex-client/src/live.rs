use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

pub type ConnectFuture = Pin<Box<dyn Future<Output = Result<IpcClient, IpcError>> + Send>>;
pub type Connector = dyn Fn() -> ConnectFuture + Send + Sync;

pub struct IpcClient {
    id: String,
}

impl IpcClient {
    pub fn new(id: &str) -> Self {
        Self {
            id: normalize_id(id),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn dead_client_debug(&self) -> String {
        format!("dead-client:{}", self.id)
    }
}

pub struct IpcError {
    message: String,
}

impl IpcError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub struct ClientRequest {
    payload: String,
}

impl ClientRequest {
    pub fn new(raw: &str) -> Self {
        Self {
            payload: normalize_id(raw),
        }
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}

pub struct ClientResponse {
    body: String,
}

impl ClientResponse {
    pub fn ok(body: &str) -> Self {
        Self {
            body: body.to_string(),
        }
    }

    pub fn missing(label: &str) -> Self {
        Self::ok(&format!("missing:{label}"))
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn dead_response(&self) -> String {
        format!("dead-response:{}", self.body)
    }
}

pub trait RequestHandler {
    fn handle(&self, request: ClientRequest) -> ClientResponse;

    fn dead_handler_method(&self) -> String {
        "dead-handler".to_string()
    }
}

pub struct ReconnectController {
    id: String,
    connector: Arc<Connector>,
    handler: Arc<RwLock<Option<Arc<dyn RequestHandler>>>>,
}

impl ReconnectController {
    pub fn new(id: &str, connector: Arc<Connector>) -> Self {
        Self {
            id: normalize_id(id),
            connector,
            handler: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_handler(&self, handler: Arc<dyn RequestHandler>) {
        *self
            .handler
            .write()
            .expect("request handler lock should not poison") = Some(handler);
    }

    pub fn summarize(&self, raw: &str) -> String {
        let _future = (self.connector)();
        let request = ClientRequest::new(raw);
        let response = self
            .handler
            .read()
            .expect("request handler lock should not poison")
            .as_ref()
            .map(|handler| handler.handle(request))
            .unwrap_or_else(|| ClientResponse::missing(&self.id));
        format!("{}:{}", self.id, response.body())
    }

    pub fn dead_controller_debug(&self) -> String {
        format!("dead-controller:{}", self.id)
    }
}

struct EchoHandler;

impl RequestHandler for EchoHandler {
    fn handle(&self, request: ClientRequest) -> ClientResponse {
        ClientResponse::ok(request.payload())
    }
}

fn normalize_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub fn selected_reconnect(raw: &str) -> String {
    let label = normalize_id(raw);
    let connector: Arc<Connector> = Arc::new(move || {
        let id = label.clone();
        Box::pin(async move { Ok(IpcClient::new(&id)) })
    });
    let controller = ReconnectController::new(raw, connector);
    controller.set_handler(Arc::new(EchoHandler));
    controller.summarize(raw)
}

pub fn dead_live_reconnect(raw: &str) -> String {
    ReconnectController::new(
        raw,
        Arc::new(|| Box::pin(async { Err(IpcError::new("dead")) })),
    )
    .dead_controller_debug()
}
