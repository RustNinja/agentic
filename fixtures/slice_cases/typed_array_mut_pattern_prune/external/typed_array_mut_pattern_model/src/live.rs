pub struct TypedArrayMutPatternPayload {
    value: String,
}

impl TypedArrayMutPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-array-mut-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-array-mut-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-array-mut-pattern:{}", self.value)
    }
}

pub fn selected_typed_array_mut_pattern(raw: &str) -> String {
    let mut values = [
        TypedArrayMutPatternPayload::new(raw),
        TypedArrayMutPatternPayload::new("tail"),
    ];
    let [payload, _other]: &mut [TypedArrayMutPatternPayload; 2] = &mut values;
    payload.bump_and_render()
}

pub fn dead_live_typed_array_mut_pattern(raw: &str) -> String {
    TypedArrayMutPatternPayload::new(raw).unused_label()
}
