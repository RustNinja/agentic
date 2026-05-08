pub struct WireValue {
    payload: String,
}

pub struct DomainValue {
    payload: String,
}

impl From<&str> for WireValue {
    fn from(value: &str) -> Self {
        Self {
            payload: value.trim().to_string(),
        }
    }
}

impl From<WireValue> for DomainValue {
    fn from(value: WireValue) -> Self {
        Self {
            payload: value.payload,
        }
    }
}

impl From<DomainValue> for WireValue {
    fn from(value: DomainValue) -> Self {
        Self {
            payload: value.payload,
        }
    }
}

impl WireValue {
    pub fn render(&self) -> String {
        format!("wire:{}", self.payload)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-wire:{}", self.payload)
    }
}

pub fn selected_wire(raw: &str) -> String {
    let wire = WireValue::from(raw);
    let domain = DomainValue::from(wire);
    WireValue::from(domain).render()
}

pub fn dead_live_wire(raw: &str) -> String {
    WireValue::from(raw).dead_method()
}
