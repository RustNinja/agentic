pub enum DispatchMethod {
    Start(StartParams),
    Stop(StopParams),
    List(ListParams),
    Notify(DeadNotification),
    Unknown { method: String },
}

impl DispatchMethod {
    pub fn from_wire(method: &str, payload: &str) -> Self {
        match method {
            "start" => Self::Start(StartParams {
                label: payload.trim().to_string(),
            }),
            other => Self::Unknown {
                method: other.to_string(),
            },
        }
    }

    pub fn render(self) -> String {
        match self {
            Self::Start(params) => params.render(),
            Self::Unknown { method } => format!("unknown:{method}"),
            _ => "unsupported".to_string(),
        }
    }

    pub fn dead_method(self) -> String {
        "dead-dispatch".to_string()
    }
}

pub struct StartParams {
    label: String,
}

impl StartParams {
    pub fn render(self) -> String {
        format!("start:{}", self.label)
    }
}

pub struct StopParams {
    reason: String,
}

impl StopParams {
    pub fn render(self) -> String {
        format!("stop:{}", self.reason)
    }
}

pub struct ListParams {
    cursor: Option<String>,
}

impl ListParams {
    pub fn render(self) -> String {
        format!("list:{}", self.cursor.unwrap_or_else(|| "first".to_string()))
    }
}

pub enum DeadNotification {
    Started { label: String },
    Stopped(StopParams),
}

impl DeadNotification {
    pub fn render(self) -> String {
        match self {
            Self::Started { label } => format!("notify-started:{label}"),
            Self::Stopped(params) => params.render(),
        }
    }
}

pub fn selected_dispatch(method: &str, payload: &str) -> String {
    DispatchMethod::from_wire(method, payload).render()
}

pub fn dead_live_dispatch(raw: &str) -> String {
    format!("dead-live-dispatch:{raw}")
}
