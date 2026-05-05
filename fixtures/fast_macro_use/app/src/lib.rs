use macro_helpers::{
    fixture_attr, fixture_constructor, fixture_export, FixtureDerive, FixtureEnum, FixtureError,
    FixtureObject, FixtureRecord,
};
use opensourced::opensourced;
use shared::{
    fixture_value as selected_value,
    prelude::{exported_nested, SharedAlias, SharedMode, FEATURE_FLAG},
    SharedRecord, SHARED_STATIC,
};
use std::{
    fmt,
    future::Future,
    pin::Pin,
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
};
use util::{make_util, DescribeValue, Transform, Useful, UtilValue};

use crate::grouped::{dead_grouped as local_shadow, live_grouped};
use crate::macro_support::bridge_try;
use crate::reexports::reexported_nested;
use crate::removed::dead_fn as selected_shadow;

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

#[allow(unused_imports)]
mod dependency_barrel {
    pub mod records {
        pub use shared::dead_shared as dead_shared_barrel;
        pub use shared::{SharedMode as BarrelMode, SharedRecord as BarrelRecord};
    }

    pub mod helpers {
        use super::records::{BarrelMode, BarrelRecord};

        pub fn score_record(record: BarrelRecord, mode: BarrelMode) -> u32 {
            record.value + mode.score()
        }

        #[allow(dead_code)]
        pub fn dead_helper() -> u32 {
            99
        }
    }

    pub use helpers::dead_helper as dead_barrel_helper;
    pub use helpers::score_record;
    pub use records::*;
}

mod inline_child {
    use super::{RootDto, SharedMode};

    pub fn child_score(dto: &RootDto) -> u32 {
        SharedMode::Fast(dto.value).score()
    }
}

#[allow(dead_code)]
mod serde_helpers {
    pub mod wire {
        pub fn serialize(value: &u32) -> u32 {
            *value
        }

        pub fn deserialize(value: u32) -> u32 {
            value
        }

        pub fn dead_wire_helper(value: u32) -> u32 {
            value + 99
        }
    }

    pub fn parse_label(value: String) -> String {
        value
    }

    pub fn empty() -> String {
        String::new()
    }

    pub fn dead_parse_label(value: String) -> String {
        value
    }

    pub fn dead_empty() -> String {
        "dead".to_string()
    }
}

#[allow(dead_code, unused_imports)]
mod runtime_shared {
    use super::SharedAlias;
    use std::sync::OnceLock;

    static RUNTIME_SEED: OnceLock<SharedAlias> = OnceLock::new();

    pub fn shared_runtime() -> SharedAlias {
        *RUNTIME_SEED.get_or_init(|| 7)
    }

    pub fn shared_mobile_client() -> SharedAlias {
        shared_runtime() + 1
    }

    pub fn shared_mobile_client_if_initialized() -> Option<SharedAlias> {
        RUNTIME_SEED.get().copied()
    }

    macro_rules! blocking_async {
        ($expr:expr) => {
            $crate::runtime_shared::shared_mobile_client() + $expr
        };
    }

    pub(crate) use blocking_async;
}

#[cfg_attr(any(unix, windows), macro_helpers::fixture_attr(shared::helper_marker))]
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

#[allow(unused_imports, unused_macros)]
mod macro_support {
    macro_rules! bridge_try {
        ($expr:expr) => {
            $expr.map_err($crate::ClientError::from)
        };
    }

    macro_rules! unused_bridge {
        () => {
            99
        };
    }

    pub(crate) use bridge_try;
    pub(crate) use unused_bridge;
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
const BINARY_GUIDE: &[u8] = include_bytes!("guidelines/binary.bin");
const DEAD_GUIDE: &str = include_str!("guidelines/dead.md");
const LOCAL_PATTERN_TAG: SharedAlias = 29;

fn asset_score() -> SharedAlias {
    (CORE_GUIDE.len() + EXTRA_GUIDE.len() + CONCAT_GUIDE.len() + BINARY_GUIDE.len()) as SharedAlias
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
#[opensourced]
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

pub trait LocalBound {
    fn raw(&self) -> SharedAlias;
}

#[derive(Clone)]
pub struct GenericValue(SharedAlias);

impl LocalBound for GenericValue {
    fn raw(&self) -> SharedAlias {
        self.0
    }
}

pub struct GenericEnvelope<T: LocalBound>
where
    T: Clone,
{
    value: T,
}

impl<T> GenericEnvelope<T>
where
    T: LocalBound + Clone,
{
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn score(&self) -> SharedAlias {
        self.value.raw()
    }
}

pub trait GenericApi<T: LocalBound>
where
    T: Clone,
{
    fn pack(value: T) -> GenericEnvelope<T>;
}

pub struct GenericCodec;

impl GenericApi<GenericValue> for GenericCodec {
    fn pack(value: GenericValue) -> GenericEnvelope<GenericValue> {
        GenericEnvelope::new(value)
    }
}

#[derive(Clone, FixtureRecord)]
#[fixture_serde(rename_all = "snake_case")]
pub struct LitterWireDto<T: LocalBound>
where
    T: Clone,
{
    #[fixture_serde(
        with = "crate::serde_helpers::wire",
        serialize_with = "crate::serde_helpers::wire::serialize",
        deserialize_with = "crate::serde_helpers::wire::deserialize"
    )]
    value: SharedAlias,
    #[fixture_serde(
        default = "crate::serde_helpers::empty",
        deserialize_with = "serde_helpers::parse_label"
    )]
    label: String,
    payload: T,
}

impl<T> LitterWireDto<T>
where
    T: LocalBound + Clone,
{
    pub fn new(value: SharedAlias, label: String, payload: T) -> Self {
        Self {
            value,
            label,
            payload,
        }
    }

    pub fn score(&self) -> SharedAlias {
        self.value + self.label.len() as SharedAlias + self.payload.raw()
    }
}

#[derive(Clone, macro_helpers::FixtureRecord)]
#[fixture_serde(rename_all = "snake_case")]
#[opensourced]
pub struct SplitRecord {
    pub value: SharedAlias,
}

#[derive(Clone, Debug, FixtureError)]
#[fixture_error(display = "client error")]
#[opensourced]
pub enum ClientError {
    #[fixture_error(display = "wire")]
    Wire { code: SharedAlias },
    #[fixture_error(display = "missing")]
    Missing,
}

impl From<WireError> for ClientError {
    fn from(error: WireError) -> Self {
        Self::Wire { code: error.code() }
    }
}

#[derive(Clone, Copy, Default, FixtureEnum)]
#[fixture_serde(rename_all = "snake_case")]
pub enum BoundaryStatus {
    #[default]
    Ready,
    Paused,
    Closed,
}

impl BoundaryStatus {
    pub fn weight(self) -> SharedAlias {
        match self {
            BoundaryStatus::Ready => 3,
            BoundaryStatus::Paused => 5,
            BoundaryStatus::Closed => 7,
        }
    }
}

#[derive(Clone, FixtureEnum)]
#[fixture_serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Dto {
        dto: WireDto,
        status: BoundaryStatus,
    },
    Error(ClientError),
    Empty,
}

impl ServerEvent {
    pub fn score(&self) -> SharedAlias {
        match self {
            ServerEvent::Dto { dto, status } => dto.score() + status.weight(),
            ServerEvent::Error(ClientError::Wire { code }) => *code,
            ServerEvent::Error(ClientError::Missing) | ServerEvent::Empty => 0,
        }
    }
}

#[derive(Clone, FixtureRecord)]
#[fixture_serde(transparent)]
pub struct PublicEnvelope {
    pub event: ServerEvent,
}

pub struct InternalEnvelope {
    event: ServerEvent,
}

impl From<PublicEnvelope> for InternalEnvelope {
    fn from(envelope: PublicEnvelope) -> Self {
        Self {
            event: envelope.event,
        }
    }
}

impl From<InternalEnvelope> for PublicEnvelope {
    fn from(envelope: InternalEnvelope) -> Self {
        Self {
            event: envelope.event,
        }
    }
}

#[derive(FixtureObject)]
#[opensourced]
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

macro_helpers::fixture_setup!(BridgeObject);

#[fixture_export(callback_interface)]
#[opensourced]
pub trait BridgeCallback {
    fn adjust(&self, value: SharedAlias) -> SharedAlias;
}

#[derive(FixtureObject)]
#[opensourced]
pub struct RemotePathObject {
    path: String,
}

#[fixture_export]
impl RemotePathObject {
    #[fixture_constructor]
    pub fn new(path: String) -> Arc<Self> {
        Arc::new(Self { path })
    }

    pub fn path_len(&self) -> SharedAlias {
        self.path.len() as SharedAlias
    }
}

macro_helpers::fixture_setup!(RemotePathObject);

#[derive(FixtureObject)]
pub struct LayeredClient {
    dto: WireDto,
    token: DisplayToken,
}

impl LayeredClient {
    fn private_seed(value: SharedAlias) -> DisplayToken {
        DisplayToken::new(value + 1)
    }

    #[allow(dead_code)]
    fn dead_private_seed() -> DisplayToken {
        DisplayToken::new(99)
    }
}

#[fixture_export(async_runtime = "fixture")]
impl LayeredClient {
    #[fixture_constructor]
    pub fn new(value: SharedAlias) -> Arc<Self> {
        Arc::new(Self {
            dto: WireDto {
                label: Some(format!("layer-{value}")),
                mode: SharedMode::Fast(value),
            },
            token: Self::private_seed(value),
        })
    }

    pub async fn compute(&self) -> SharedAlias {
        runtime_shared::blocking_async!(self.dto.score() + self.token.score())
    }

    pub fn event(&self) -> ServerEvent {
        ServerEvent::Dto {
            dto: self.dto.clone(),
            status: BoundaryStatus::Ready,
        }
    }
}

macro_helpers::fixture_setup!(LayeredClient);

mod private_facade {
    use super::{SharedAlias, WireDto};

    #[derive(Clone, macro_helpers::FixtureRecord)]
    #[fixture_serde(rename_all = "snake_case")]
    pub struct FacadeRecord {
        dto: WireDto,
    }

    impl FacadeRecord {
        pub fn new(dto: WireDto) -> Self {
            Self { dto }
        }

        pub fn score(&self) -> SharedAlias {
            self.dto.score()
        }
    }

    #[derive(macro_helpers::FixtureObject)]
    pub struct FacadeObject {
        record: FacadeRecord,
    }

    #[macro_helpers::fixture_export]
    impl FacadeObject {
        #[macro_helpers::fixture_constructor]
        pub fn new(dto: WireDto) -> Self {
            Self {
                record: FacadeRecord::new(dto),
            }
        }

        pub fn score(&self) -> SharedAlias {
            self.record.score()
        }
    }

    pub fn dead_facade() -> SharedAlias {
        99
    }
}

pub use private_facade::dead_facade as dead_facade_alias;
pub use private_facade::FacadeObject;

#[allow(dead_code, unused_imports)]
mod ffi_barrel {
    pub mod app_store {
        use super::super::{SharedAlias, WireDto};
        use std::collections::VecDeque;

        #[derive(Clone, macro_helpers::FixtureRecord)]
        #[fixture_serde(rename_all = "snake_case")]
        pub struct AppStoreUpdateRecord {
            dto: WireDto,
        }

        impl AppStoreUpdateRecord {
            pub fn score(&self) -> SharedAlias {
                self.dto.score()
            }
        }

        #[derive(macro_helpers::FixtureObject)]
        pub struct AppStoreSubscription {
            updates: VecDeque<AppStoreUpdateRecord>,
        }

        #[macro_helpers::fixture_export]
        impl AppStoreSubscription {
            pub fn next_update(&mut self) -> Option<AppStoreUpdateRecord> {
                self.updates.pop_front()
            }

            pub fn dead_poll(&mut self) -> SharedAlias {
                99
            }
        }

        #[derive(macro_helpers::FixtureObject)]
        pub struct AppStore {
            dto: WireDto,
        }

        #[macro_helpers::fixture_export]
        impl AppStore {
            #[macro_helpers::fixture_constructor]
            pub fn new(dto: WireDto) -> Self {
                Self { dto }
            }

            pub fn subscribe_updates(&self) -> AppStoreSubscription {
                let mut updates = VecDeque::new();
                updates.push_back(AppStoreUpdateRecord {
                    dto: self.dto.clone(),
                });
                AppStoreSubscription { updates }
            }

            pub fn snapshot(&self) -> SharedAlias {
                self.dto.score()
            }

            pub fn dead_start_turn(&self) -> SharedAlias {
                99
            }
        }
    }

    pub mod reconnect {
        pub struct ReconnectController;

        impl ReconnectController {
            pub fn reconnect(&self) -> u32 {
                1
            }
        }
    }

    pub mod alleycat {
        pub struct AlleycatBridge;
    }

    pub use alleycat::AlleycatBridge;
    pub use app_store::{AppStore, AppStoreSubscription};
    pub use reconnect::ReconnectController;
}

#[fixture_export(callback_interface)]
pub trait ReconnectCallback {
    fn reconnect(&self, label: String) -> SharedAlias;
}

#[derive(FixtureObject)]
pub struct CallbackRegistry {
    callback: Arc<dyn ReconnectCallback + Send + Sync>,
    decisions: Mutex<Vec<SharedAlias>>,
}

#[fixture_export]
impl CallbackRegistry {
    #[fixture_constructor]
    pub fn new(callback: Box<dyn ReconnectCallback + Send + Sync>) -> Arc<Self> {
        Arc::new(Self {
            callback: Arc::from(callback),
            decisions: Mutex::new(Vec::new()),
        })
    }

    pub fn record(&self, label: String) -> SharedAlias {
        let value = self.callback.reconnect(label);
        self.decisions
            .lock()
            .expect("fixture mutex should lock")
            .push(value);
        value
    }
}

pub type BoxDecisionFuture = Pin<Box<dyn Future<Output = bool> + Send>>;
pub type DecisionCallback = Arc<dyn Fn(&str) -> BoxDecisionFuture + Send + Sync>;

static DECISION_CALLBACK: OnceLock<DecisionCallback> = OnceLock::new();

pub trait RemoteTransport {
    fn send(&self, event: ServerEvent) -> BoxDecisionFuture;
}

pub struct TransportBundle {
    transport: Arc<dyn RemoteTransport + Send + Sync>,
    keepalive: Option<Arc<dyn Send + Sync>>,
    client: Arc<LayeredClient>,
}

impl TransportBundle {
    pub fn new(
        transport: Arc<dyn RemoteTransport + Send + Sync>,
        keepalive: Option<Arc<dyn Send + Sync>>,
        client: Arc<LayeredClient>,
    ) -> Self {
        Self {
            transport,
            keepalive,
            client,
        }
    }

    pub fn has_keepalive(&self) -> bool {
        self.keepalive.is_some()
    }

    pub fn client_event_score(&self) -> SharedAlias {
        let _ = &self.transport;
        self.client.event().score()
    }
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

fn registry_bridge_score(
    callback: Box<dyn ReconnectCallback + Send + Sync>,
    label: String,
) -> SharedAlias {
    let registry = CallbackRegistry::new(callback);
    let remote = RemotePathObject::new(label.clone());
    registry.record(label) + remote.path_len()
}

fn macro_reexport_conversion(dto: WireDto) -> Result<RootDto, ClientError> {
    bridge_try!(RootDto::try_from(dto))
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

pub fn open_registry_bridge(
    callback: Box<dyn ReconnectCallback + Send + Sync>,
    label: String,
) -> SharedAlias {
    registry_bridge_score(callback, label)
}

pub fn open_decision_callback(callback: DecisionCallback, label: &str) -> SharedAlias {
    let _ = DECISION_CALLBACK.set(callback.clone());
    let future = callback(label);
    drop(future);
    if DECISION_CALLBACK.get().is_some() {
        1
    } else {
        0
    }
}

pub fn open_macro_helper_reexport(dto: WireDto) -> SharedAlias {
    macro_reexport_conversion(dto)
        .map(|dto| dto.value)
        .unwrap_or_else(|error| match error {
            ClientError::Wire { code } => code,
            ClientError::Missing => 0,
        })
}

pub fn open_conversion_roundtrip(event: ServerEvent) -> SharedAlias {
    let public = PublicEnvelope { event };
    let internal = InternalEnvelope::from(public);
    let public = PublicEnvelope::from(internal);
    public.event.score()
}

pub fn open_facade_reexport(dto: WireDto) -> SharedAlias {
    FacadeObject::new(dto).score()
}

pub fn open_app_store_subscription(dto: WireDto) -> SharedAlias {
    let store = ffi_barrel::AppStore::new(dto);
    let mut subscription = store.subscribe_updates();
    subscription
        .next_update()
        .map(|record| record.score())
        .unwrap_or(0)
}

pub fn open_layered_client(value: SharedAlias) -> SharedAlias {
    let client = LayeredClient::new(value);
    client.event().score()
}

pub fn open_transport_bundle(
    transport: Arc<dyn RemoteTransport + Send + Sync>,
    keepalive: Option<Arc<dyn Send + Sync>>,
    value: SharedAlias,
) -> TransportBundle {
    TransportBundle::new(transport, keepalive, LayeredClient::new(value))
}

#[opensourced]
pub fn open_generic_edges(value: SharedAlias) -> SharedAlias {
    let envelope = GenericEnvelope::new(GenericValue(value));
    let packed = <GenericCodec as GenericApi<GenericValue>>::pack(GenericValue(1));
    let wire = LitterWireDto::new(value, format!("generic-{value}"), GenericValue(2));
    envelope.score() + packed.score() + wire.score()
}

#[opensourced]
pub fn open_dependency_barrel(value: SharedAlias) -> SharedAlias {
    let record = dependency_barrel::BarrelRecord { value };
    let mode = dependency_barrel::BarrelMode::Fast(record.value);
    dependency_barrel::score_record(record, mode)
}

#[fixture_export(async_runtime = "fixture")]
#[opensourced]
pub async fn open_async_macro_use(input: u32) -> u32 {
    async_bridge_value(input).await
        + exported_bridge(
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
