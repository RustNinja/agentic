#[cfg_attr(feature = "ffi", derive(Debug, uniffi::Record))]
pub struct LiveCfgRecord {
    pub label: String,
    #[cfg_attr(feature = "ffi", uniffi(default = "LiveCfgStatus::Unknown"))]
    pub status: LiveCfgStatus,
}

#[cfg_attr(feature = "ffi", derive(Debug, uniffi::Enum))]
pub enum LiveCfgStatus {
    Ready,
    Unknown,
}

#[cfg_attr(feature = "ffi", cfg_attr(feature = "bindings", uniffi::export))]
impl LiveCfgRecord {
    pub fn render(&self) -> String {
        format!("{}:{}", self.label, self.status.label())
    }
}

impl LiveCfgRecord {
    pub fn dead_method(&self) -> String {
        format!("dead-cfg:{}", self.label)
    }
}

impl LiveCfgStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unknown => "unknown",
        }
    }
}

pub fn selected_cfg_record(raw: &str) -> LiveCfgRecord {
    LiveCfgRecord {
        label: raw.trim().to_string(),
        status: LiveCfgStatus::Ready,
    }
}

pub fn dead_live_cfg_record(raw: &str) -> String {
    selected_cfg_record(raw).dead_method()
}

