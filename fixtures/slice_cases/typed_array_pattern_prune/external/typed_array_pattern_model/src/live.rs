pub struct TypedArrayPatternPayload {
    value: String,
}

impl TypedArrayPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-array-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-array-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-array-pattern:{}", self.value)
    }
}

pub fn selected_typed_array_pattern(raw: &str) -> String {
    let [payload, _other]: [TypedArrayPatternPayload; 2] = [
        TypedArrayPatternPayload::new(raw),
        TypedArrayPatternPayload::new("tail"),
    ];
    payload.render_label()
}

pub fn dead_live_typed_array_pattern(raw: &str) -> String {
    TypedArrayPatternPayload::new(raw).unused_label()
}
