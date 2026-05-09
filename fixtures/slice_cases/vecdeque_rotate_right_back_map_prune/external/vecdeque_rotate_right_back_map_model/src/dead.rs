pub struct DeadVecdequeRotateRightBackMapItem {
    value: String,
}

impl DeadVecdequeRotateRightBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-rotate-right-back-map:{}", self.value)
    }
}

pub fn dead_vecdeque_rotate_right_back_map(raw: &str) -> String {
    DeadVecdequeRotateRightBackMapItem::new(raw).dead_method()
}
