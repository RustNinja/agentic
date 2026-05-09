pub struct DeadIteratorFindMapNestedOkQuestionItem {
    value: String,
}

impl DeadIteratorFindMapNestedOkQuestionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-find-map-nested-ok-question:{}", self.value)
    }
}

pub fn dead_iterator_find_map_nested_ok_question(raw: &str) -> String {
    DeadIteratorFindMapNestedOkQuestionItem::new(raw).dead_method()
}
