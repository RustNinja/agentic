use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecAsMutSliceIterMutItem {
    value: String,
}

impl VecAsMutSliceIterMutItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn other() -> Self {
        Self::new("other")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value.contains("live")
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-live");
        self
    }

    pub fn sort_key(&self) -> usize {
        self.value.len()
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn render_label(&self) -> String {
        format!("vec-as-mut-slice-iter-mut:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vec-as-mut-slice-iter-mut:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-as-mut-slice-iter-mut:{}", self.value)
    }
}

fn vec_as_mut_slice_iter_mut_entries(raw: &str) -> Vec<VecAsMutSliceIterMutItem> {
    vec![VecAsMutSliceIterMutItem::live(), VecAsMutSliceIterMutItem::other(), VecAsMutSliceIterMutItem::new(raw)]
}

pub fn selected_vec_as_mut_slice_iter_mut(raw: &str) -> String {
    let mut entries = vec_as_mut_slice_iter_mut_entries(raw);
    entries
        .as_mut_slice()
        .iter_mut()
        .map(|item| item.bump().render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_as_mut_slice_iter_mut(raw: &str) -> String {
    VecAsMutSliceIterMutItem::new(raw).dead_method()
}
