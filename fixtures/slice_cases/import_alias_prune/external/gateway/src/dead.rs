use domain::dead::{build_dead_model, DeadModel};

pub struct DeadGateway {
    model: DeadModel,
}

impl DeadGateway {
    pub fn new(raw: &str) -> Self {
        Self {
            model: build_dead_model(raw),
        }
    }

    pub fn render(self) -> String {
        format!("dead-gateway:{}", self.model.render_dead())
    }
}

pub fn dead_report(raw: &str) -> String {
    DeadGateway::new(raw).render()
}
