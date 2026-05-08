pub struct NewtypeInner(String);

impl NewtypeInner {
    pub fn render(&self) -> String {
        format!("inner:{}", self.0)
    }

    pub fn dead_inner_method(&self) -> String {
        format!("dead-inner:{}", self.0)
    }
}

pub struct NewtypeRecord(NewtypeInner);

impl NewtypeRecord {
    pub fn render(&self) -> String {
        self.0.render()
    }

    pub fn dead_method(&self) -> String {
        self.0.dead_inner_method()
    }
}

pub fn selected_newtype(raw: &str) -> NewtypeRecord {
    NewtypeRecord(NewtypeInner(raw.trim().to_string()))
}

pub fn dead_live_newtype(raw: &str) -> String {
    selected_newtype(raw).dead_method()
}
