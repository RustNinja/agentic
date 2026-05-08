use leaf::LeafRecord as PublicLeaf;

macro_rules! render_leaf {
    ($leaf:expr) => {
        $leaf.render()
    };
}

pub struct AdapterRecord {
    leaf: PublicLeaf,
}

impl AdapterRecord {
    pub fn new(value: &str) -> Self {
        Self {
            leaf: PublicLeaf::new(value),
        }
    }

    pub fn render(self) -> String {
        format!("adapter:{}", render_leaf!(self.leaf))
    }

    pub fn dead_method(self) -> String {
        format!("dead-adapter:{}", self.leaf.render())
    }
}

pub fn selected_bridge(value: &str) -> String {
    AdapterRecord::new(value).render()
}

pub fn dead_live_bridge(value: &str) -> String {
    AdapterRecord::new(value).dead_method()
}

