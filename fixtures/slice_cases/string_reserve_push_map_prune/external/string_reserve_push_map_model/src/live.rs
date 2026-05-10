pub struct StringReservePushMapPayload {
    value: String,
}

impl StringReservePushMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("string-reserve-push-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("string-reserve-push-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-string-reserve-push-map:{}", self.value)
    }
}

pub fn selected_string_reserve_push_map(raw: &str) -> String {
    let mut value = String::with_capacity(raw.len() + 4);
    value.reserve(4);
    value.push_str(raw);
    value.push('!');
    StringReservePushMapPayload::new(&value).render_label()
}

pub fn dead_live_string_reserve_push_map(raw: &str) -> String {
    StringReservePushMapPayload::new(raw).unused_label()
}
