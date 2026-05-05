use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use opensource_core::{generate, GenerateOptions};

#[test]
fn slices_fast_macro_use_fixture_and_compiles() {
    let fixture = repo_root().join("fixtures/fast_macro_use");
    let output = temp_path("fast-macro-use-output");
    let target_dir = temp_path("fast-macro-use-target");

    let report = generate(GenerateOptions {
        workspace_root: fixture,
        output_root: output.clone(),
    })
    .expect("fast macro/use fixture should reduce");

    assert_eq!(
        report.packages,
        ["fast_macro_app", "macro_helpers", "shared", "util"]
    );
    assert_eq!(report.production.status, "hazards_detected");
    assert!(
        report
            .production
            .hazards
            .iter()
            .any(|hazard| hazard.code == "source_include_macros" && hazard.severity == "error"),
        "fast fixture should report retained generated source includes: {:?}",
        report.production.hazards
    );
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "dynamic_callback_boundaries" && hazard.severity == "warning"
    }));

    let app = read(output.join("fast_macro_app/src/lib.rs"));
    assert!(app.contains("pub fn open_macro_use_entry"));
    assert!(app.contains("pub async fn open_async_macro_use"));
    assert!(app.contains("pub fn open_callback_bridge"));
    assert!(app.contains("FixtureDerive"));
    assert!(app.contains("FixtureRecord"));
    assert!(app.contains("FixtureEnum"));
    assert!(app.contains("FixtureError"));
    assert!(app.contains("FixtureObject"));
    assert!(app.contains("fixture_attr"));
    assert!(app.contains("fixture_export"));
    assert!(app.contains("fixture_constructor"));
    assert!(app.contains("use crate::generated_types"));
    assert!(app.contains("include!(\"generated.rs\")"));
    assert!(app.contains("mod inline_child"));
    assert!(app.contains("mod platform_bridge"));
    assert!(app.contains("mod reexports"));
    assert!(app.contains("reexported_nested"));
    assert!(app.contains("SharedMode"));
    assert!(app.contains("SharedAlias"));
    assert!(app.contains("FEATURE_FLAG"));
    assert!(app.contains("SHARED_STATIC"));
    assert!(app.contains("UtilValue::BONUS"));
    assert!(app.contains("impl From"));
    assert!(app.contains("for RootDto"));
    assert!(app.contains("pub struct WireDto"));
    assert!(app.contains("pub enum WireKind"));
    assert!(app.contains("pub enum WireEvent"));
    assert!(app.contains("pub struct WireSlot"));
    assert!(app.contains("impl TryFrom"));
    assert!(app.contains("declare_wire_error"));
    assert!(app.contains("WireError"));
    assert!(app.contains("declare_macro_pair"));
    assert!(app.contains("MacroPair"));
    assert!(app.contains("macro_pair"));
    assert!(app.contains("include_str!(\"guidelines/core.md\")"));
    assert!(app.contains("include_bytes!(\"guidelines/binary.bin\")"));
    assert!(app.contains("include_str!(\"../assets/extra.txt\")"));
    assert!(app.contains("include_str!(concat!(\"guidelines/\", \"concat.md\"))"));
    assert!(app.contains("mod cfg_matrix"));
    assert!(app.contains("LOCAL_PATTERN_TAG"));
    assert!(app.contains("pub struct SplitRecord"));
    assert!(app.contains("macro_helpers::FixtureRecord"));
    assert!(app.contains("pub enum ClientError"));
    assert!(app.contains("pub fn open_trait_edges"));
    assert!(app.contains("pub fn open_cfg_asset_bridge"));
    assert!(app.contains("pub struct DisplayToken"));
    assert!(app.contains("impl fmt::Display for DisplayToken"));
    assert!(app.contains("pub trait EdgeCodec"));
    assert!(app.contains("impl EdgeCodec<WireDto> for WireCodec"));
    assert!(app.contains("pub struct BridgeObject"));
    assert!(app.contains("pub struct RemotePathObject"));
    assert!(app.contains("pub trait BridgeCallback"));
    assert!(app.contains("fn async_bridge_value"));
    assert!(app.contains("pub fn open_dependency_barrel"));
    assert!(app.contains("mod dependency_barrel"));
    assert!(app.contains("BarrelRecord"));
    assert!(app.contains("BarrelMode"));
    assert!(!app.contains("pub struct CallbackRegistry"));
    assert!(!app.contains("pub fn open_decision_callback"));
    assert!(!app.contains("pub fn open_registry_bridge"));
    assert!(!app.contains("dead_fn as selected_shadow"));
    assert!(!app.contains("dead_grouped as local_shadow"));
    assert!(!app.contains("dead_reexport"));
    assert!(!app.contains("dead_platform"));
    assert!(!app.contains("dynamic_registry"));
    assert!(!app.contains("unused_macro"));
    assert!(!app.contains("DEAD_GUIDE"));
    assert!(!app.contains("dead_public_api"));
    assert_include_assets(&output, &app);

    let shared = read(output.join("shared/src/lib.rs"));
    assert!(shared.contains("pub type SharedAlias"));
    assert!(shared.contains("pub const FEATURE_FLAG"));
    assert!(shared.contains("pub static SHARED_STATIC"));
    assert!(shared.contains("pub enum SharedMode"));
    assert!(shared.contains("pub mod prelude"));
    assert!(shared.contains("pub fn fixture_value"));
    assert!(shared.contains("pub fn helper_marker"));
    assert!(!shared.contains("dead_shared"));
    assert!(!shared.contains("dead_nested"));
    assert!(!shared.contains("DeadEnum"));
    assert!(!shared.contains("dead_prelude"));

    let util = read(output.join("util/src/lib.rs"));
    assert!(util.contains("pub trait Useful"));
    assert!(util.contains("pub trait Transform"));
    assert!(util.contains("pub trait DescribeValue"));
    assert!(util.contains("Transform"));
    assert!(util.contains("for UtilValue"));
    assert!(util.contains("type Output"));
    assert!(util.contains("const BONUS"));
    assert!(util.contains("impl FromStr for UtilValue"));
    assert!(util.contains("impl fmt::Display for UtilValue"));
    assert!(util.contains("impl Deref for UtilValue"));
    assert!(util.contains("pub fn make_util"));
    assert!(!util.contains("dead_util"));

    let macro_helpers = read(output.join("macro_helpers/src/lib.rs"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureRecord"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureEnum"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureObject"));
    assert!(macro_helpers.contains("proc_macro_derive(FixtureError"));
    assert!(macro_helpers.contains("pub fn fixture_export"));

    assert_cargo_check(&output, &target_dir, &app);
}

#[test]
fn slices_fast_fixture_from_multiple_root_angles() {
    let cases = [
        SliceAngle {
            label: "macro-root",
            roots: &["open_macro_use_entry"],
            present: &[
                "pub fn open_macro_use_entry",
                "include!(\"generated.rs\")",
                "declare_wire_error",
                "pub enum WireEvent",
                "pub struct WireSlot",
                "impl TryFrom",
                "declare_macro_pair",
                "macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include_bytes!(\"guidelines/binary.bin\")",
                "mod cfg_matrix",
                "pub struct BridgeObject",
            ],
            absent: &[
                "pub async fn open_async_macro_use",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "pub fn open_callback_bridge",
                "pub trait BridgeCallback",
                "pub trait EdgeCodec",
                "dynamic_registry",
                "dead_public_api",
            ],
        },
        SliceAngle {
            label: "async-root",
            roots: &["open_async_macro_use"],
            present: &[
                "pub async fn open_async_macro_use",
                "fn async_bridge_value",
                "pub struct WireDto",
                "pub enum WireKind",
                "pub fn exported_bridge",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "include!(\"generated.rs\")",
                "declare_wire_error",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "pub struct BridgeObject",
                "pub fn open_callback_bridge",
                "pub trait BridgeCallback",
                "pub trait EdgeCodec",
            ],
        },
        SliceAngle {
            label: "callback-root",
            roots: &["open_callback_bridge"],
            present: &[
                "pub fn open_callback_bridge",
                "pub trait BridgeCallback",
                "mod platform_bridge",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "pub async fn open_async_macro_use",
                "pub struct WireDto",
                "declare_wire_error",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include!(\"generated.rs\")",
                "pub struct BridgeObject",
            ],
        },
        SliceAngle {
            label: "wire-item-root",
            roots: &["WireDto"],
            present: &["pub struct WireDto", "SharedMode"],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "pub async fn open_async_macro_use",
                "pub fn open_callback_bridge",
                "declare_wire_error",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include!(\"generated.rs\")",
                "pub struct BridgeObject",
            ],
        },
        SliceAngle {
            label: "event-enum-root",
            roots: &["WireEvent"],
            present: &[
                "pub enum WireEvent",
                "Record { dto: WireDto }",
                "Failed(WireError)",
                "pub struct WireDto",
                "declare_wire_error",
                "FixtureEnum",
                "FixtureRecord",
                "FixtureError",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "pub async fn open_async_macro_use",
                "pub fn open_callback_bridge",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include!(\"generated.rs\")",
                "pub struct BridgeObject",
                "pub trait EdgeCodec",
            ],
        },
        SliceAngle {
            label: "trait-item-root",
            roots: &["BridgeCallback"],
            present: &["pub trait BridgeCallback", "fn adjust"],
            absent: &[
                "pub fn open_callback_bridge",
                "pub struct WireDto",
                "declare_wire_error",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include!(\"generated.rs\")",
                "pub struct BridgeObject",
            ],
        },
        SliceAngle {
            label: "object-item-root",
            roots: &["BridgeObject"],
            present: &[
                "pub struct BridgeObject",
                "FixtureObject",
                "fixture_export",
                "fixture_constructor",
                "pub fn new",
                "pub fn value",
                "pub struct RootDto",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "pub async fn open_async_macro_use",
                "pub fn open_callback_bridge",
                "pub enum WireEvent",
                "declare_wire_error",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include!(\"generated.rs\")",
                "pub trait EdgeCodec",
                "pub trait BridgeCallback",
            ],
        },
        SliceAngle {
            label: "split-record-root",
            roots: &["SplitRecord"],
            present: &[
                "pub struct SplitRecord",
                "macro_helpers::FixtureRecord",
                "fixture_serde",
                "SharedAlias",
            ],
            absent: &[
                "use macro_helpers::FixtureRecord",
                "pub fn open_macro_use_entry",
                "pub enum ClientError",
                "pub struct RemotePathObject",
                "pub struct CallbackRegistry",
                "pub enum WireEvent",
                "declare_wire_error",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "client-error-root",
            roots: &["ClientError"],
            present: &[
                "pub enum ClientError",
                "FixtureError",
                "fixture_error",
                "Wire { code: SharedAlias }",
                "Missing",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub struct SplitRecord",
                "pub struct RemotePathObject",
                "pub struct CallbackRegistry",
                "pub enum WireEvent",
                "declare_wire_error",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "remote-object-root",
            roots: &["RemotePathObject"],
            present: &[
                "pub struct RemotePathObject",
                "FixtureObject",
                "fixture_export",
                "fixture_constructor",
                "pub fn new",
                "Arc<Self>",
                "pub fn path_len",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub struct BridgeObject",
                "pub struct CallbackRegistry",
                "pub trait ReconnectCallback",
                "pub enum WireEvent",
                "declare_wire_error",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "registry-object-root",
            roots: &["CallbackRegistry"],
            present: &[
                "pub struct CallbackRegistry",
                "pub trait ReconnectCallback",
                "fn reconnect",
                "Arc<dyn ReconnectCallback",
                "Mutex<Vec<SharedAlias>>",
                "fixture_constructor",
                "pub fn record",
            ],
            absent: &[
                "pub fn open_registry_bridge",
                "pub struct RemotePathObject",
                "pub fn open_macro_use_entry",
                "pub struct BridgeObject",
                "pub enum WireEvent",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "registry-bridge-root",
            roots: &["open_registry_bridge"],
            present: &[
                "pub fn open_registry_bridge",
                "fn registry_bridge_score",
                "pub struct CallbackRegistry",
                "pub trait ReconnectCallback",
                "pub struct RemotePathObject",
                "Arc<dyn ReconnectCallback",
                "fixture_constructor",
                "pub fn path_len",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_decision_callback",
                "pub struct BridgeObject",
                "pub enum WireEvent",
                "declare_wire_error",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "decision-callback-root",
            roots: &["open_decision_callback"],
            present: &[
                "pub fn open_decision_callback",
                "pub type BoxDecisionFuture",
                "pub type DecisionCallback",
                "static DECISION_CALLBACK",
                "dyn Future<Output = bool>",
                "dyn Fn",
                "OnceLock",
            ],
            absent: &[
                "pub fn open_registry_bridge",
                "pub struct CallbackRegistry",
                "pub struct RemotePathObject",
                "pub fn open_macro_use_entry",
                "pub struct BridgeObject",
                "pub enum WireEvent",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "server-event-root",
            roots: &["ServerEvent"],
            present: &[
                "pub enum ServerEvent",
                "Dto {",
                "status: BoundaryStatus",
                "pub enum BoundaryStatus",
                "pub struct WireDto",
                "pub enum ClientError",
                "FixtureEnum",
                "FixtureRecord",
                "FixtureError",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub struct PublicEnvelope",
                "pub struct FacadeObject",
                "pub struct LayeredClient",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "macro-helper-reexport-root",
            roots: &["open_macro_helper_reexport"],
            present: &[
                "pub fn open_macro_helper_reexport",
                "fn macro_reexport_conversion",
                "macro_rules! bridge_try",
                "pub(crate) use bridge_try",
                "impl TryFrom<WireDto> for RootDto",
                "pub enum ClientError",
            ],
            absent: &[
                "unused_bridge",
                "pub fn open_conversion_roundtrip",
                "pub struct FacadeObject",
                "pub struct LayeredClient",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "conversion-roundtrip-root",
            roots: &["open_conversion_roundtrip"],
            present: &[
                "pub fn open_conversion_roundtrip",
                "pub struct PublicEnvelope",
                "pub struct InternalEnvelope",
                "impl From<PublicEnvelope> for InternalEnvelope",
                "impl From<InternalEnvelope> for PublicEnvelope",
                "pub enum ServerEvent",
            ],
            absent: &[
                "pub fn open_macro_helper_reexport",
                "pub struct FacadeObject",
                "pub struct LayeredClient",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "facade-reexport-root",
            roots: &["open_facade_reexport"],
            present: &[
                "pub fn open_facade_reexport",
                "mod private_facade",
                "pub struct FacadeObject",
                "pub struct FacadeRecord",
                "pub use private_facade::FacadeObject",
                "macro_helpers::FixtureObject",
                "macro_helpers::fixture_export",
            ],
            absent: &[
                "dead_facade",
                "dead_facade_alias",
                "pub fn open_macro_helper_reexport",
                "pub struct LayeredClient",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "facade-object-item-root",
            roots: &["FacadeObject"],
            present: &[
                "mod private_facade",
                "pub struct FacadeObject",
                "pub struct FacadeRecord",
                "macro_helpers::FixtureObject",
                "macro_helpers::fixture_constructor",
                "pub fn new",
                "pub fn score",
            ],
            absent: &[
                "pub fn open_facade_reexport",
                "dead_facade",
                "pub struct LayeredClient",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "layered-object-root",
            roots: &["LayeredClient"],
            present: &[
                "pub struct LayeredClient",
                "fn private_seed",
                "pub async fn compute",
                "pub fn event",
                "pub enum ServerEvent",
                "pub enum BoundaryStatus",
                "macro_helpers::fixture_setup!(LayeredClient)",
            ],
            absent: &[
                "dead_private_seed",
                "pub fn open_layered_client",
                "pub struct FacadeObject",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "layered-fn-root",
            roots: &["open_layered_client"],
            present: &[
                "pub fn open_layered_client",
                "pub struct LayeredClient",
                "pub fn new",
                "pub fn event",
                "pub enum ServerEvent",
            ],
            absent: &[
                "dead_private_seed",
                "pub struct FacadeObject",
                "pub fn open_transport_bundle",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "transport-bundle-root",
            roots: &["open_transport_bundle"],
            present: &[
                "pub fn open_transport_bundle",
                "pub trait RemoteTransport",
                "pub struct TransportBundle",
                "Arc<dyn RemoteTransport",
                "Option<Arc<dyn Send + Sync>>",
                "pub struct LayeredClient",
                "pub struct WireDto",
            ],
            absent: &[
                "pub fn open_decision_callback",
                "pub fn open_registry_bridge",
                "pub struct FacadeObject",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "generic-edges-root",
            roots: &["open_generic_edges"],
            present: &[
                "pub fn open_generic_edges",
                "pub trait LocalBound",
                "pub struct GenericValue",
                "pub struct GenericEnvelope<T: LocalBound>",
                "impl<T> GenericEnvelope<T>",
                "T: LocalBound + Clone",
                "pub trait GenericApi<T: LocalBound>",
                "impl GenericApi<GenericValue> for GenericCodec",
            ],
            absent: &[
                "pub fn open_transport_bundle",
                "pub struct LayeredClient",
                "pub struct FacadeObject",
                "pub fn open_macro_use_entry",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "dependency-barrel-root",
            roots: &["open_dependency_barrel"],
            present: &[
                "pub fn open_dependency_barrel",
                "mod dependency_barrel",
                "pub mod records",
                "pub use records::*",
                "BarrelRecord",
                "BarrelMode",
                "pub use helpers::score_record",
                "pub fn score_record",
            ],
            absent: &[
                "dead_shared_barrel",
                "dead_barrel_helper",
                "dead_helper",
                "pub fn open_generic_edges",
                "pub struct LayeredClient",
                "pub struct FacadeObject",
                "pub fn open_macro_use_entry",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "trait-edges-root",
            roots: &["open_trait_edges"],
            present: &[
                "pub fn open_trait_edges",
                "fn trait_edge_score",
                "pub trait EdgeCodec",
                "impl EdgeCodec<WireDto> for WireCodec",
                "pub struct DisplayToken",
                "impl fmt::Display for DisplayToken",
                "UtilValue::from_str",
                "DescribeValue",
                "WireCodec::encode",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_cfg_asset_bridge",
                "pub async fn open_async_macro_use",
                "pub fn open_callback_bridge",
                "declare_macro_pair",
                "include_str!(\"guidelines/core.md\")",
                "include!(\"generated.rs\")",
                "pub struct BridgeObject",
            ],
        },
        SliceAngle {
            label: "cfg-asset-root",
            roots: &["open_cfg_asset_bridge"],
            present: &[
                "pub fn open_cfg_asset_bridge",
                "mod cfg_matrix",
                "include_str!(\"guidelines/core.md\")",
                "include_bytes!(\"guidelines/binary.bin\")",
                "include_str!(\"../assets/extra.txt\")",
                "include_str!(concat!(\"guidelines/\", \"concat.md\"))",
                "LOCAL_PATTERN_TAG",
            ],
            absent: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub async fn open_async_macro_use",
                "pub fn open_callback_bridge",
                "declare_wire_error",
                "declare_macro_pair",
                "include!(\"generated.rs\")",
                "pub struct BridgeObject",
                "pub trait EdgeCodec",
            ],
        },
        SliceAngle {
            label: "codec-trait-item-root",
            roots: &["EdgeCodec"],
            present: &["pub trait EdgeCodec", "type Encoded", "const OFFSET"],
            absent: &[
                "pub fn open_trait_edges",
                "impl EdgeCodec<WireDto> for WireCodec",
                "pub struct WireDto",
                "pub struct DisplayToken",
                "include_str!(\"guidelines/core.md\")",
            ],
        },
        SliceAngle {
            label: "macro-async-roots",
            roots: &["open_macro_use_entry", "open_async_macro_use"],
            present: &[
                "pub fn open_macro_use_entry",
                "pub async fn open_async_macro_use",
                "declare_wire_error",
                "declare_macro_pair",
                "pub enum WireEvent",
                "pub struct WireDto",
                "include_str!(\"guidelines/core.md\")",
            ],
            absent: &[
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "pub fn open_callback_bridge",
                "pub trait BridgeCallback",
            ],
        },
        SliceAngle {
            label: "macro-trait-cfg-roots",
            roots: &[
                "open_macro_use_entry",
                "open_trait_edges",
                "open_cfg_asset_bridge",
            ],
            present: &[
                "pub fn open_macro_use_entry",
                "pub fn open_trait_edges",
                "pub fn open_cfg_asset_bridge",
                "declare_macro_pair",
                "pub trait EdgeCodec",
                "impl EdgeCodec<WireDto> for WireCodec",
                "include_str!(\"guidelines/core.md\")",
                "mod cfg_matrix",
            ],
            absent: &["pub fn open_callback_bridge", "pub trait BridgeCallback"],
        },
    ];

    for case in cases {
        let fixture = fixture_with_selected_roots(case.label, case.roots);
        let output = temp_path(&format!("fast-macro-use-{}-output", case.label));
        let target_dir = temp_path(&format!("fast-macro-use-{}-target", case.label));

        let report = generate(GenerateOptions {
            workspace_root: fixture,
            output_root: output.clone(),
        })
        .unwrap_or_else(|error| panic!("{} should reduce: {error}", case.label));
        let root_names = report
            .roots
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        for root in case.roots {
            assert!(
                root_names
                    .iter()
                    .any(|name| { name.ends_with(root) || name.contains(&format!("::{root}(")) }),
                "{} should include root {root}; roots: {:?}",
                case.label,
                root_names
            );
        }

        let app = read(output.join("fast_macro_app/src/lib.rs"));
        for expected in case.present {
            assert!(
                app.contains(expected),
                "{} expected app slice to contain {expected:?}\n{}",
                case.label,
                app
            );
        }
        for forbidden in case.absent {
            assert!(
                !app.contains(forbidden),
                "{} expected app slice to prune {forbidden:?}\n{}",
                case.label,
                app
            );
        }
        assert!(!app.contains("dead_public_api"));
        assert!(!app.contains("dynamic_registry"));
        assert!(!app.contains("DEAD_GUIDE"));
        assert_include_assets(&output, &app);
        assert_cargo_check(&output, &target_dir, &app);
    }
}

#[test]
fn flags_fast_out_dir_generated_rust_as_production_blocker() {
    let fixture = fixture_with_selected_roots("out-dir-generated", &[]);
    add_out_dir_generated_root(&fixture);

    let output = temp_path("fast-out-dir-generated-output");
    let target_dir = temp_path("fast-out-dir-generated-target");
    let report = generate(GenerateOptions {
        workspace_root: fixture,
        output_root: output.clone(),
    })
    .expect("out-dir fixture should reduce");

    assert_eq!(report.production.status, "hazards_detected");
    assert!(report
        .production
        .hazards
        .iter()
        .any(|hazard| { hazard.code == "retained_build_scripts" && hazard.severity == "error" }));
    assert!(report.production.hazards.iter().any(|hazard| {
        hazard.code == "out_dir_source_include_macros" && hazard.severity == "error"
    }));

    let app = read(output.join("fast_macro_app/src/lib.rs"));
    assert!(app.contains("pub fn open_out_dir_generated"));
    assert!(app.contains("include!(concat!(env!(\"OUT_DIR\"), \"/fast_generated.rs\"))"));
    assert!(!app.contains("pub fn open_macro_use_entry"));
    assert!(!app.contains("pub fn open_trait_edges"));
    assert_cargo_check(&output, &target_dir, &app);
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

struct SliceAngle<'a> {
    label: &'a str,
    roots: &'a [&'a str],
    present: &'a [&'a str],
    absent: &'a [&'a str],
}

fn assert_cargo_check(output: &Path, target_dir: &Path, app: &str) {
    let cargo_check = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(output)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("cargo check should start");
    assert!(
        cargo_check.status.success(),
        "generated fast macro/use slice did not compile\nstatus: {}\nstdout:\n{}\nstderr:\n{}\napp/src/lib.rs:\n{}",
        cargo_check.status,
        String::from_utf8_lossy(&cargo_check.stdout),
        String::from_utf8_lossy(&cargo_check.stderr),
        app,
    );
}

fn assert_include_assets(output: &Path, app: &str) {
    if app.contains("include_str!(\"guidelines/core.md\")") {
        assert!(output
            .join("fast_macro_app/src/guidelines/core.md")
            .exists());
        assert!(output
            .join("fast_macro_app/src/guidelines/concat.md")
            .exists());
        assert!(output
            .join("fast_macro_app/src/guidelines/binary.bin")
            .exists());
        assert!(output.join("fast_macro_app/assets/extra.txt").exists());
        assert!(
            !output
                .join("fast_macro_app/src/guidelines/dead.md")
                .exists(),
            "dead include asset should not be copied"
        );
    }
}

fn fixture_with_selected_roots(label: &str, selected_roots: &[&str]) -> PathBuf {
    let source = repo_root().join("fixtures/fast_macro_use");
    let fixture = temp_path(&format!("fast-macro-use-{label}-fixture"));
    copy_dir(&source, &fixture);
    rewrite_copied_fixture_manifest(&fixture);
    rewrite_root_markers(&fixture.join("app/src/lib.rs"), selected_roots);
    fixture
}

fn add_out_dir_generated_root(fixture: &Path) {
    let manifest = fixture.join("app/Cargo.toml");
    let manifest_text = read(&manifest);
    let manifest_text = manifest_text.replace(
        "license.workspace = true\n\n[dependencies]",
        "license.workspace = true\nbuild = \"build.rs\"\n\n[dependencies]",
    );
    fs::write(&manifest, manifest_text).expect("out-dir fixture manifest should be writable");

    fs::write(
        fixture.join("app/build.rs"),
        r#"use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    fs::write(
        out_dir.join("fast_generated.rs"),
        "pub fn build_generated_value() -> u32 { 37 }\n",
    )
    .expect("generated source should be writable");
}
"#,
    )
    .expect("out-dir fixture build script should be writable");

    let lib = fixture.join("app/src/lib.rs");
    let mut lib_text = read(&lib);
    lib_text.push_str(
        r#"
mod out_dir_generated {
    include!(concat!(env!("OUT_DIR"), "/fast_generated.rs"));
}

#[opensourced]
pub fn open_out_dir_generated() -> u32 {
    out_dir_generated::build_generated_value()
}
"#,
    );
    fs::write(lib, lib_text).expect("out-dir fixture lib should be writable");
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("destination directory should be created");
    for entry in fs::read_dir(source).expect("source directory should be readable") {
        let entry = entry.expect("source entry should be readable");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("file type should be readable")
            .is_dir()
        {
            if entry.file_name() == "target" {
                continue;
            }
            copy_dir(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("fixture file should copy");
        }
    }
}

fn rewrite_copied_fixture_manifest(fixture: &Path) {
    let manifest = fixture.join("Cargo.toml");
    let text = read(&manifest);
    let opensourced_path = repo_root().join("crates/opensourced");
    let rewritten = text.replace(
        "opensourced = { path = \"../../crates/opensourced\" }",
        &format!("opensourced = {{ path = {:?} }}", opensourced_path),
    );
    fs::write(manifest, rewritten).expect("copied fixture manifest should be writable");
}

fn rewrite_root_markers(lib: &Path, selected_roots: &[&str]) {
    let selected_roots = selected_roots.iter().copied().collect::<BTreeSet<_>>();
    let mut rewritten = String::new();

    for line in read(lib).lines() {
        if line.trim() == "#[opensourced]" {
            continue;
        }
        if let Some(name) = declared_item_name(line) {
            if selected_roots.contains(name.as_str()) {
                rewritten.push_str("#[opensourced]\n");
            }
        }
        rewritten.push_str(line);
        rewritten.push('\n');
    }

    fs::write(lib, rewritten).expect("selected-root fixture should be writable");
}

fn declared_item_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if let Some(name) = name_after_keyword(trimmed, "fn ") {
        return Some(name);
    }
    for keyword in ["struct ", "enum ", "trait "] {
        if let Some(name) = name_after_keyword(trimmed, keyword) {
            return Some(name);
        }
    }
    None
}

fn name_after_keyword(line: &str, keyword: &str) -> Option<String> {
    let start = line.find(keyword)? + keyword.len();
    let tail = &line[start..];
    let name = tail
        .chars()
        .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn temp_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("opensourced-{label}-{unique}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
