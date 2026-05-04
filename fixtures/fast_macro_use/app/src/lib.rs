use macro_helpers::{
    fixture_attr, fixture_constructor, fixture_export, FixtureDerive, FixtureEnum, FixtureError,
    FixtureObject, FixtureRecord,
};
use opensourced::opensourced;
use std::{fmt, str::FromStr};
use shared::{
    fixture_value as selected_value,
    prelude::{exported_nested, SharedAlias, SharedMode, FEATURE_FLAG},
    SharedRecord, SHARED_STATIC,
};
use util::{make_util, DescribeValue, Transform, Useful, UtilValue};

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

#[cfg_attr(
    any(unix, windows),
    macro_helpers::fixture_attr(shared::helper_marker)
)]
mod platform_bridge {
    pub fn platform_value() -> u32 {
        17
    }

    #[allow(dead_code)]
    pub fn dead_platform() -> u32 {
        99
    }
}

#[cfg(any(target_os = "macos", target_os = "linux", windows))]
mod cfg_matrix {
    pub fn cfg_value() -> u32 {
        19
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
mod cfg_matrix {
    pub fn cfg_value() -> u32 {
        0
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

macro_rules! declare_wire_error {
    ($name:ident) => {
        #[derive(Clone, FixtureError)]
        #[fixture_error(display = "wire error")]
        pub struct $name {
            code: SharedAlias,
        }

        impl $name {
            pub fn new(code: SharedAlias) -> Self {
                Self { code }
            }

            pub fn code(&self) -> SharedAlias {
                self.code
            }
        }
    };
}

declare_wire_error!(WireError);

macro_rules! declare_macro_pair {
    ($name:ident, $builder:ident) => {
        #[derive(Clone, FixtureRecord)]
        #[fixture_serde(rename_all = "snake_case")]
        pub struct $name {
            value: SharedAlias,
        }

        impl $name {
            pub fn new(value: SharedAlias) -> Self {
                Self { value }
            }

            pub fn value(&self) -> SharedAlias {
                self.value
            }
        }

        pub fn $builder(value: SharedAlias) -> $name {
            $name::new(value)
        }
    };
}

declare_macro_pair!(MacroPair, macro_pair);

const CORE_GUIDE: &str = include_str!("guidelines/core.md");
const EXTRA_GUIDE: &str = include_str!("../assets/extra.txt");
const CONCAT_GUIDE: &str = include_str!(concat!("guidelines/", "concat.md"));
const DEAD_GUIDE: &str = include_str!("guidelines/dead.md");
const LOCAL_PATTERN_TAG: SharedAlias = 29;

fn asset_score() -> SharedAlias {
    (CORE_GUIDE.len() + EXTRA_GUIDE.len() + CONCAT_GUIDE.len()) as SharedAlias
}

fn pattern_score(value: SharedAlias) -> SharedAlias {
    match value {
        FEATURE_FLAG => 5,
        LOCAL_PATTERN_TAG => 7,
        _ => 1,
    }
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

impl From<SharedRecord> for RootDto {
    fn from(record: SharedRecord) -> Self {
        Self::new(record.value)
    }
}

#[derive(Clone, FixtureRecord)]
#[fixture_serde(rename_all = "camelCase")]
#[opensourced]
pub struct WireDto {
    #[fixture_serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub mode: SharedMode,
}

impl WireDto {
    pub fn score(&self) -> SharedAlias {
        let label_len = self.label.as_ref().map(|value| value.len()).unwrap_or(0) as u32;
        self.mode.score() + label_len
    }
}

#[derive(Clone, Copy, FixtureEnum)]
#[fixture_serde(rename_all = "kebab-case")]
pub enum WireKind {
    Created,
    Updated,
}

impl WireKind {
    pub fn weight(self) -> SharedAlias {
        match self {
            WireKind::Created => 1,
            WireKind::Updated => 2,
        }
    }
}

#[derive(Clone, FixtureEnum)]
#[fixture_serde(tag = "type", rename_all = "snake_case")]
pub enum WireEvent {
    Record { dto: WireDto },
    Failed(WireError),
    Empty,
}

impl WireEvent {
    pub fn score(&self) -> SharedAlias {
        match self {
            WireEvent::Record { dto } => dto.score(),
            WireEvent::Failed(error) => error.code(),
            WireEvent::Empty => 0,
        }
    }
}

#[opensourced]
pub struct DisplayToken(SharedAlias);

impl DisplayToken {
    pub fn new(value: SharedAlias) -> Self {
        Self(value)
    }

    pub fn score(&self) -> SharedAlias {
        self.to_string().len() as SharedAlias
    }
}

impl fmt::Display for DisplayToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "display-{}", self.0)
    }
}

pub struct WireSlot<const N: usize> {
    value: SharedAlias,
}

impl<const N: usize> WireSlot<N> {
    pub fn new(value: SharedAlias) -> Self {
        Self { value }
    }

    pub fn folded(&self) -> SharedAlias {
        self.value + N as SharedAlias
    }
}

#[opensourced]
pub trait EdgeCodec<T> {
    type Encoded;

    const OFFSET: SharedAlias;

    fn encode(value: T) -> Self::Encoded;
}

pub struct WireCodec;

impl EdgeCodec<WireDto> for WireCodec {
    type Encoded = SharedAlias;

    const OFFSET: SharedAlias = 31;

    fn encode(value: WireDto) -> Self::Encoded {
        value.score() + Self::OFFSET
    }
}

#[derive(FixtureObject)]
pub struct BridgeObject {
    dto: RootDto,
}

#[fixture_export]
impl BridgeObject {
    #[fixture_constructor]
    pub fn new(value: u32) -> Self {
        Self {
            dto: RootDto::new(value),
        }
    }

    pub fn value(&self) -> SharedAlias {
        self.dto.value
    }
}

#[fixture_export(callback_interface)]
#[opensourced]
pub trait BridgeCallback {
    fn adjust(&self, value: SharedAlias) -> SharedAlias;
}

#[fixture_export(shared::helper_marker)]
pub fn exported_bridge(dto: WireDto, kind: WireKind) -> SharedAlias {
    dto.score() + kind.weight()
}

impl TryFrom<WireDto> for RootDto {
    type Error = WireError;

    fn try_from(dto: WireDto) -> Result<Self, Self::Error> {
        match dto.mode {
            SharedMode::Fast(value) => Ok(Self::new(value)),
            SharedMode::Slow => Err(WireError::new(1)),
        }
    }
}

async fn async_bridge_value(input: SharedAlias) -> SharedAlias {
    let dto = RootDto::from(SharedRecord::new(input));
    dto.value + platform_bridge::platform_value()
}

fn trait_edge_score(input: SharedAlias, wire: WireDto) -> SharedAlias {
    let parsed = UtilValue::from_str("8").expect("static util value should parse");
    let parsed_via_trait: UtilValue = "9".parse().expect("static util value should parse");
    let encoded: <WireCodec as EdgeCodec<WireDto>>::Encoded = WireCodec::encode(wire);
    let display = DisplayToken::new(input);

    encoded
        + <WireCodec as EdgeCodec<WireDto>>::OFFSET
        + parsed.transform(input)
        + <UtilValue as Transform<SharedAlias>>::transform(&parsed_via_trait, input)
        + *parsed
        + parsed_via_trait.describe_value().len() as SharedAlias
        + <UtilValue as DescribeValue>::LABEL.len() as SharedAlias
        + display.score()
}

#[allow(dead_code)]
mod dynamic_registry {
    pub trait InternalWorker {
        fn work(&self) -> u32;
    }

    pub struct LocalWorker;

    impl InternalWorker for LocalWorker {
        fn work(&self) -> u32 {
            91
        }
    }

    pub struct Registry {
        worker: Box<dyn InternalWorker>,
    }

    impl Registry {
        pub fn new() -> Self {
            Self {
                worker: Box::new(LocalWorker),
            }
        }

        pub fn run(&self) -> u32 {
            self.worker.work()
        }
    }

    pub fn dead_registry() -> Registry {
        Registry::new()
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
    let dto = RootDto::from(record);
    let mode = SharedMode::Fast(record.value);
    let alias_value: SharedAlias = selected_value();
    let const_mix = FEATURE_FLAG + SHARED_STATIC + exported_nested() + reexported_nested();
    let wire = WireDto {
        label: Some(format!("feature-{FEATURE_FLAG}")),
        mode,
    };
    let converted = RootDto::try_from(wire.clone())
        .map(|dto| dto.value)
        .unwrap_or_else(|error| error.code());
    let event = WireEvent::Record { dto: wire.clone() };
    let slot = WireSlot::<3>::new(converted);
    let pair = macro_pair(alias_value);
    let bridge_object = BridgeObject::new(input);

    local_sum!(
        shared::shared_macro!(dto.value)
            + live_grouped()
            + util.useful()
            + util.transform(alias_value)
            + UtilValue::BONUS
            + mode.score()
            + inline_child::child_score(&dto)
            + exported_bridge(wire, WireKind::Created)
            + event.score(),
        generated.value()
            + selected_shadow
            + local_shadow
            + closure(1)
            + via_match
            + via_loop
            + alias_value
            + const_mix
            + converted
            + slot.folded()
            + pair.value()
            + cfg_matrix::cfg_value()
            + asset_score()
            + pattern_score(alias_value)
            + bridge_object.value()
    )
}

#[opensourced]
pub fn open_trait_edges(input: SharedAlias) -> SharedAlias {
    trait_edge_score(
        input,
        WireDto {
            label: Some(DisplayToken::new(input).to_string()),
            mode: SharedMode::Fast(input),
        },
    )
}

#[opensourced]
pub fn open_cfg_asset_bridge(input: SharedAlias) -> SharedAlias {
    cfg_matrix::cfg_value() + asset_score() + pattern_score(input)
}

#[fixture_export(async_runtime = "fixture")]
#[opensourced]
pub async fn open_async_macro_use(input: u32) -> u32 {
    async_bridge_value(input).await + exported_bridge(
        WireDto {
            label: None,
            mode: SharedMode::Fast(input),
        },
        WireKind::Updated,
    )
}

#[opensourced]
pub fn open_callback_bridge(
    callback: &dyn BridgeCallback,
    adjust: fn(SharedAlias) -> SharedAlias,
    value: SharedAlias,
) -> SharedAlias {
    callback.adjust(value) + adjust(value) + platform_bridge::platform_value()
}

pub fn dead_public_api() -> u32 {
    selected_shadow() + local_shadow() + unused_macro!() + DEAD_GUIDE.len() as u32
}
