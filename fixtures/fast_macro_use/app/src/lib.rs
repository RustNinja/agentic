use macro_helpers::{fixture_attr, FixtureDerive};
use opensourced::opensourced;
use shared::{
    fixture_value as selected_value,
    prelude::{exported_nested, SharedAlias, SharedMode, FEATURE_FLAG},
    SharedRecord, SHARED_STATIC,
};
use util::{make_util, Useful, UtilValue};

use crate::grouped::{dead_grouped as local_shadow, live_grouped};
use crate::removed::dead_fn as selected_shadow;
use crate::reexports::reexported_nested;

pub(crate) mod generated_types {
    include!("generated.rs");
}

mod generated_bridge {
    use crate::generated_types;

    pub fn generated() -> generated_types::GeneratedMessage {
        generated_types::GeneratedMessage::new(5)
    }
}

mod grouped {
    pub fn live_grouped() -> u32 {
        1
    }

    pub fn dead_grouped() -> u32 {
        99
    }
}

mod removed {
    pub fn dead_fn() -> u32 {
        99
    }
}

#[allow(unused_imports)]
mod reexports {
    pub use shared::dead_shared as dead_reexport;
    pub use shared::nested::nested_value as reexported_nested;
}

mod inline_child {
    use super::{RootDto, SharedMode};

    pub fn child_score(dto: &RootDto) -> u32 {
        SharedMode::Fast(dto.value).score()
    }
}

macro_rules! local_sum {
    ($left:expr, $right:expr) => {
        $left + $right
    };
}

macro_rules! unused_macro {
    () => {
        0
    };
}

#[derive(Default, FixtureDerive)]
#[fixture_helper(path = "shared::helper_marker")]
pub struct RootDto {
    value: u32,
}

impl RootDto {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[fixture_attr(shared::helper_marker)]
#[opensourced]
pub fn open_macro_use_entry(input: u32) -> u32 {
    let selected_shadow = 1;
    let local_shadow = {
        let local_shadow = 2;
        local_shadow
    };
    let closure = |selected_value: u32| selected_value + 1;
    let via_match = match Some(3) {
        Some(selected_value) => selected_value,
        None => 0,
    };
    let mut via_loop = 0;
    for selected_value in [4] {
        via_loop += selected_value;
    }

    let generated = generated_bridge::generated();
    let util: util::UtilValue = make_util(input);
    let record = SharedRecord::new(selected_value());
    let dto = RootDto::new(record.value);
    let mode = SharedMode::Fast(record.value);
    let alias_value: SharedAlias = selected_value();
    let const_mix = FEATURE_FLAG + SHARED_STATIC + exported_nested() + reexported_nested();

    local_sum!(
        shared::shared_macro!(dto.value)
            + live_grouped()
            + util.useful()
            + UtilValue::BONUS
            + mode.score()
            + inline_child::child_score(&dto),
        generated.value()
            + selected_shadow
            + local_shadow
            + closure(1)
            + via_match
            + via_loop
            + alias_value
            + const_mix
    )
}

pub fn dead_public_api() -> u32 {
    selected_shadow() + local_shadow() + unused_macro!()
}
