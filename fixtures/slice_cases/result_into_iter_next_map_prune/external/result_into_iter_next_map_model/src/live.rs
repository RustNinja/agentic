pub struct ResultIntoIterNextMapPayload {
    value: String,
}

impl ResultIntoIterNextMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("result-into-iter-next-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("result-into-iter-next-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-result-into-iter-next-map:{}", self.value)
    }
}

pub fn selected_result_into_iter_next_map(raw: &str) -> String {
    Ok::<_, ()>(ResultIntoIterNextMapPayload::new(raw))
        .into_iter()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_result_into_iter_next_map(raw: &str) -> String {
    ResultIntoIterNextMapPayload::new(raw).unused_label()
}
