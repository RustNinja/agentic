pub struct DeadOptionAsMutSliceIterMutItem {
    value: String,
}

impl DeadOptionAsMutSliceIterMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-as-mut-slice-iter-mut:{}", self.value)
    }
}

pub fn dead_option_as_mut_slice_iter_mut(raw: &str) -> String {
    DeadOptionAsMutSliceIterMutItem::new(raw).render()
}
