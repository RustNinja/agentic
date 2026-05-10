pub struct TypedSliceEnumTuplePatternPayload {
    value: String,
}

impl TypedSliceEnumTuplePatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-slice-enum-tuple-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-slice-enum-tuple-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-slice-enum-tuple-pattern:{}", self.value)
    }
}

pub enum TypedSliceEnumTuplePatternEnvelope {
    Live(TypedSliceEnumTuplePatternPayload),
    Dead,
}

pub fn selected_typed_slice_enum_tuple_pattern(raw: &str) -> String {
    let [TypedSliceEnumTuplePatternEnvelope::Live(payload), ..]: [TypedSliceEnumTuplePatternEnvelope; 1] = [TypedSliceEnumTuplePatternEnvelope::Live(TypedSliceEnumTuplePatternPayload::new(raw))] else {
        return "typed-slice-enum-tuple-pattern:missing".to_string();
    };
    payload.render_label()
}

pub fn dead_live_typed_slice_enum_tuple_pattern(raw: &str) -> String {
    TypedSliceEnumTuplePatternPayload::new(raw).unused_label()
}
