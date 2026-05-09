pub struct DeadVecDequeMakeContiguousSortItem {
    value: String,
}

impl DeadVecDequeMakeContiguousSortItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-make-contiguous-sort:{}", self.value)
    }
}

pub fn dead_vecdeque_make_contiguous_sort(raw: &str) -> String {
    DeadVecDequeMakeContiguousSortItem::new(raw).dead_method()
}
