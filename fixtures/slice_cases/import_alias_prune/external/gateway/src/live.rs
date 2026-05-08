use domain::{
    dead::{build_dead_model as make_dead_model, DeadModel as DeadAlias},
    live::{build_model as make_model, LiveModel as PublicModel},
    prelude::{dead_prefix, model_prefix},
};

macro_rules! render_model {
    ($model:expr) => {
        $model.render()
    };
}

pub struct GatewayRecord {
    model: PublicModel,
}

impl GatewayRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            model: make_model(raw),
        }
    }

    pub fn render(self) -> String {
        format!("{}:{}", model_prefix(), render_model!(self.model))
    }

    pub fn dead_method(self) -> String {
        let dead: DeadAlias = make_dead_model("dead");
        format!("{}:{}", dead_prefix(), dead.render_dead())
    }
}

pub fn selected_report(raw: &str) -> String {
    GatewayRecord::new(raw).render()
}

pub fn dead_live_gateway(raw: &str) -> String {
    GatewayRecord::new(raw).dead_method()
}
