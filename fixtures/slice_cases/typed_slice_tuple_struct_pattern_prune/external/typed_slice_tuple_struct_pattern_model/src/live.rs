pub struct TypedSliceTupleStructPatternPayload {
    value: String,
}

impl TypedSliceTupleStructPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("typed-slice-tuple-struct-pattern:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("typed-slice-tuple-struct-pattern:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-typed-slice-tuple-struct-pattern:{}", self.value)
    }
}

pub struct TypedSliceTupleStructPatternEnvelope(TypedSliceTupleStructPatternPayload);

pub fn selected_typed_slice_tuple_struct_pattern(raw: &str) -> String {
    let [TypedSliceTupleStructPatternEnvelope(payload), ..]: [TypedSliceTupleStructPatternEnvelope; 1] = [TypedSliceTupleStructPatternEnvelope(TypedSliceTupleStructPatternPayload::new(raw))];
    payload.render_label()
}

pub fn dead_live_typed_slice_tuple_struct_pattern(raw: &str) -> String {
    TypedSliceTupleStructPatternPayload::new(raw).unused_label()
}
