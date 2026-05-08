#![allow(dead_code)]

macro_rules! define_generated {
    ($name:ident, $builder:ident, $prefix:literal) => {
        pub struct $name {
            label: String,
        }

        impl $name {
            pub fn new(raw: &str) -> Self {
                Self {
                    label: raw.trim().to_string(),
                }
            }

            pub fn render(self) -> String {
                format!("{}:{}", $prefix, self.label)
            }

            pub fn dead_method(self) -> String {
                format!("dead-generated:{}", self.label)
            }
        }

        pub fn $builder(raw: &str) -> $name {
            $name::new(raw)
        }
    };
}

define_generated!(LiveGenerated, build_live_generated, "live");
define_generated!(DeadGenerated, build_dead_generated, "dead");

pub fn selected_generated(raw: &str) -> String {
    build_live_generated(raw).render()
}

pub fn dead_generated(raw: &str) -> String {
    build_dead_generated(raw).dead_method()
}
