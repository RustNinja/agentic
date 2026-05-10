pub struct TypedArrayRefPatternPayload {
    value: String,
}

impl TypedArrayRefPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-array-ref-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-array-ref-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-array-ref-pattern:{}", self.value)
    }
}

pub fn selected_typed_array_ref_pattern(raw: &str) -> String {
    let [payload, _other]: &[TypedArrayRefPatternPayload; 2] = &[
        TypedArrayRefPatternPayload::new(raw),
        TypedArrayRefPatternPayload::new("tail"),
    ];
    payload.render_label()
}

pub fn dead_live_typed_array_ref_pattern(raw: &str) -> String {
    TypedArrayRefPatternPayload::new(raw).unused_label()
}
