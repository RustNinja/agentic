#[derive(Clone)]
pub struct ArrayFromFnIterPayload {
    value: String,
}

impl ArrayFromFnIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("array-from-fn-iter:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("array-from-fn-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-array-from-fn-iter:{}", self.value)
    }
}

pub fn selected_array_from_fn_iter(raw: &str) -> String {
    let payloads: [ArrayFromFnIterPayload; 2] =
        std::array::from_fn(|index| ArrayFromFnIterPayload::new(&format!("{raw}:{index}")));
    payloads
        .iter()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "array-from-fn-iter:missing".to_string())
}

pub fn dead_live_array_from_fn_iter(raw: &str) -> String {
    ArrayFromFnIterPayload::new(raw).dead_method()
}
