use macro_support::{dead_attr, DeadRecord};
use model::DeadWire;

#[derive(Clone, DeadRecord)]
#[dead_attr]
pub struct DeadApiRecord {
    wire: DeadWire,
}

impl DeadApiRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            wire: DeadWire::new(raw),
        }
    }

    pub fn render(self) -> String {
        format!("dead-api:{}", self.wire.render_dead())
    }
}

pub fn dead_wire(raw: &str) -> String {
    DeadApiRecord::new(raw).render()
}
