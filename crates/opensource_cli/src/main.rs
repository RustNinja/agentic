use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc, OnceLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use opensource_core::{
    check_workspace, direct_free_function_root_selectors,
    generate_with_analyzer_feedback_and_roots, marked_workspace_packages, preflight_workspace,
    repair_workspace, resolve_feedback_widening_roots, write_generate_report,
    write_preflight_report, write_repair_report, write_report, AnalyzerMode, CheckDiagnostic,
    CheckOptions, CheckReport, FeedbackRootResolutionEntry, FeedbackRootResolutionReport,
    GenerateOptions, GenerateReport, GenerateSession, GenerateSessionLoadProgress,
    GeneratedTargetReport, PreflightDiagnostic, PreflightOptions, PreflightReport, RepairOptions,
    RepairReport, RootId, SemanticReport,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = parse_args()?;

    if same_path(&options.workspace_root, &options.output_root) {
        return Err("output root must be different from workspace root".into());
    }

    initialize_event_log(&options)?;
    write_event_log(
        &options,
        "startup",
        "configured",
        "initialize CLI run before workspace probing",
        "event logging starts before metadata/package-scope resolution so slow startup is visible",
        event_fields(&[
            ("workspace_root", serde_json::json!(options.workspace_root)),
            ("output_root", serde_json::json!(options.output_root)),
            (
                "analyzer",
                serde_json::json!(options.analyzer_mode.as_str()),
            ),
            ("batch_roots", serde_json::json!(options.batch_roots)),
            ("root_selectors", serde_json::json!(options.root_selectors)),
        ]),
    )?;
    write_event_log(
        &options,
        "scope_resolution",
        "started",
        "resolve default validation package scope",
        "the CLI may inspect the source workspace before generation to avoid over-broad cargo checks",
        event_fields(&[(
            "cargo_check_args_before",
            serde_json::json!(options.cargo_check_args),
        )]),
    )?;
    if let Err(error) = apply_default_marked_package_scope(&mut options) {
        let reason = error.to_string();
        write_event_log(
            &options,
            "scope_resolution",
            "failed",
            "resolve default validation package scope",
            &reason,
            event_fields(&[(
                "cargo_check_args",
                serde_json::json!(options.cargo_check_args),
            )]),
        )?;
        return Err(error);
    }
    write_event_log(
        &options,
        "scope_resolution",
        "completed",
        "resolve default validation package scope",
        "default package scope is ready and generation can start",
        event_fields(&[(
            "cargo_check_args_after",
            serde_json::json!(options.cargo_check_args),
        )]),
    )?;

    if options.batch_roots {
        return run_batch_roots(&options);
    }

    write_event_log(
        &options,
        "input",
        "configured",
        "select top-level slice roots",
        if options.root_selectors.is_empty() {
            "source #[opensourced] markers define the top-level open-source surface"
        } else {
            "--root/--roots-file defines the top-level open-source surface without modifying source"
        },
        event_fields(&[
            ("workspace_root", serde_json::json!(options.workspace_root)),
            ("output_root", serde_json::json!(options.output_root)),
            (
                "analyzer",
                serde_json::json!(options.analyzer_mode.as_str()),
            ),
            ("root_selectors", serde_json::json!(options.root_selectors)),
            ("production", serde_json::json!(options.production_preset)),
        ]),
    )?;

    let mut validation = ValidationReport::new(&options);

    let baseline = if options.run_baseline_check {
        write_event_log(
            &options,
            "baseline",
            "started",
            "run source workspace cargo check",
            "source baseline distinguishes existing project failures from slicer-introduced failures",
            event_fields(&[
                ("target_dir", serde_json::json!(baseline_target_dir(&options))),
                ("cargo_args", serde_json::json!(options.cargo_check_args)),
            ]),
        )?;
        let report = run_baseline_check(&options)?;
        let report_path = baseline_report_path(&options);
        write_event_log(
            &options,
            "baseline",
            if report.success { "passed" } else { "failed" },
            "record source workspace baseline",
            if report.success {
                "source workspace cargo check passed"
            } else if options.allow_baseline_failures {
                "source workspace failed but failures are allowed as comparison baseline"
            } else {
                "source workspace failed and validation must reject before slicing"
            },
            event_fields(&[
                ("report_path", serde_json::json!(report_path)),
                ("errors", serde_json::json!(report.error_count())),
                ("warnings", serde_json::json!(report.warning_count())),
                ("duration_ms", serde_json::json!(report.duration_ms)),
            ]),
        )?;
        if !report.success && !options.allow_baseline_failures {
            write_baseline_report(&options, &report)?;
            print_baseline(&report, options.feedback_limit, Some(&report_path));
            validation.add_check_gate(
                "baseline",
                "failed",
                "source workspace failed baseline cargo check",
                &report,
                Some(report_path),
                0,
            );
            finish_validation(
                &options,
                &mut validation,
                "rejected",
                Some("source workspace failed baseline cargo check"),
            )?;
            return Err("source workspace failed baseline cargo check".into());
        }
        print_baseline(&report, options.feedback_limit, None);
        validation.add_check_gate(
            "baseline",
            if report.success {
                "passed"
            } else {
                "baseline_allowed"
            },
            if report.success {
                "source workspace cargo check passed"
            } else {
                "source workspace baseline failed but --allow-baseline-failures is enabled"
            },
            &report,
            Some(report_path),
            0,
        );
        Some(report)
    } else {
        None
    };

    write_event_log(
        &options,
        "generation",
        "started",
        "build top-down dependency closure",
        "the slicer starts at selected roots and retains only downstream dependencies needed by those roots",
        event_fields(&[
            ("analyzer", serde_json::json!(options.analyzer_mode.as_str())),
            ("output_root", serde_json::json!(options.output_root)),
        ]),
    )?;
    write_event_log(
        &options,
        "generation_analyzer",
        "started",
        "load analyzer session for selected roots",
        "single-root mode resolves roots and loads the analyzer before rendering the output workspace",
        event_fields(&[
            ("analyzer", serde_json::json!(options.analyzer_mode.as_str())),
            ("root_selectors", serde_json::json!(options.root_selectors)),
        ]),
    )?;
    let _generation_analyzer_heartbeat = start_event_heartbeat(
        &options,
        "generation_analyzer",
        "load analyzer session for selected roots",
        "single-root analyzer loading is still running before render planning starts",
        event_fields(&[
            (
                "analyzer",
                serde_json::json!(options.analyzer_mode.as_str()),
            ),
            ("root_selectors", serde_json::json!(options.root_selectors)),
        ]),
    );
    let direct_roots = if options.root_selectors.is_empty() {
        None
    } else {
        direct_free_function_root_selectors(&options.root_selectors)
    };
    let loaded = match direct_roots {
        Some(roots) => GenerateSession::load_with_selected_roots(
            &options.workspace_root,
            options.analyzer_mode,
            &roots,
        )
        .map(|session| (session, roots)),
        None if options.root_selectors.is_empty() => {
            GenerateSession::load(&options.workspace_root, options.analyzer_mode)
                .map(|session| (session, Vec::new()))
        }
        None => GenerateSession::load_with_root_selectors(
            &options.workspace_root,
            options.analyzer_mode,
            &options.root_selectors,
        ),
    };
    let (session, selected_roots) = match loaded {
        Ok(loaded) => loaded,
        Err(error) => {
            let reason = error.to_string();
            write_event_log(
                &options,
                "generation_analyzer",
                "failed",
                "load analyzer session for selected roots",
                &reason,
                event_fields(&[
                    (
                        "analyzer",
                        serde_json::json!(options.analyzer_mode.as_str()),
                    ),
                    ("root_selectors", serde_json::json!(options.root_selectors)),
                ]),
            )?;
            return Err(error);
        }
    };
    drop(_generation_analyzer_heartbeat);
    let mut analyzer_fields = session_load_fields(&session);
    analyzer_fields.extend(event_fields(&[
        (
            "analyzer",
            serde_json::json!(session.analyzer().mode.as_str()),
        ),
        ("engine", serde_json::json!(&session.analyzer().engine)),
        ("notes", serde_json::json!(&session.analyzer().notes)),
        (
            "selected_roots",
            serde_json::json!(selected_roots
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()),
        ),
    ]));
    write_event_log(
        &options,
        "generation_analyzer",
        "completed",
        "load analyzer session for selected roots",
        "the loaded session is ready to render the downstream dependency closure",
        analyzer_fields,
    )?;

    write_event_log(
        &options,
        "generation_render",
        "started",
        "render selected roots plus downstream closure",
        "rendering writes the candidate slice workspace after analyzer loading is complete",
        event_fields(&[
            ("output_root", serde_json::json!(options.output_root)),
            (
                "selected_roots",
                serde_json::json!(selected_roots
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()),
            ),
        ]),
    )?;
    let _generation_render_heartbeat = start_event_heartbeat(
        &options,
        "generation_render",
        "render selected roots plus downstream closure",
        "single-root rendering is still walking downstream dependencies and writing output files",
        event_fields(&[
            ("output_root", serde_json::json!(options.output_root)),
            (
                "selected_roots",
                serde_json::json!(selected_roots
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()),
            ),
        ]),
    );
    let report = match session.generate(options.output_root.clone(), &selected_roots, &[]) {
        Ok(report) => report,
        Err(error) => {
            let reason = error.to_string();
            write_event_log(
                &options,
                "generation_render",
                "failed",
                "render selected roots plus downstream closure",
                &reason,
                event_fields(&[
                    ("output_root", serde_json::json!(options.output_root)),
                    (
                        "selected_roots",
                        serde_json::json!(selected_roots
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()),
                    ),
                ]),
            )?;
            return Err(error);
        }
    };
    drop(_generation_render_heartbeat);
    write_event_log(
        &options,
        "generation_render",
        "completed",
        "render selected roots plus downstream closure",
        "rendering completed and the candidate slice workspace is on disk",
        event_fields(&[
            ("root_count", serde_json::json!(report.roots.len())),
            ("packages", serde_json::json!(report.packages)),
            ("files_written", serde_json::json!(report.files_written)),
            ("total_ms", serde_json::json!(report.timings.total_ms)),
            ("reduce_ms", serde_json::json!(report.timings.reduce_ms)),
            ("render_ms", serde_json::json!(report.timings.render_ms)),
        ]),
    )?;
    write_event_log(
        &options,
        "generation",
        "completed",
        "render selected roots plus downstream closure",
        "generation completed and produced a slice workspace for validation",
        event_fields(&[
            ("root_count", serde_json::json!(report.roots.len())),
            ("packages", serde_json::json!(report.packages)),
            ("files_written", serde_json::json!(report.files_written)),
            ("total_ms", serde_json::json!(report.timings.total_ms)),
            ("analyzer_ms", serde_json::json!(report.timings.analyzer_ms)),
            ("reduce_ms", serde_json::json!(report.timings.reduce_ms)),
            ("render_ms", serde_json::json!(report.timings.render_ms)),
        ]),
    )?;

    println!(
        "analyzer: {} ({})",
        report.analyzer.mode.as_str(),
        report.analyzer.engine
    );
    for note in &report.analyzer.notes {
        println!("  analyzer note: {note}");
    }
    if let Some(semantic) = &report.analyzer.semantic {
        println!(
            "  analyzer semantic files: {}/{} analyzed ({} failed, {} skipped by budget)",
            semantic.analyzed_files,
            semantic.source_files,
            semantic.failed_files,
            semantic.skipped_files
        );
        println!(
            "  analyzer budgets: files={}, method_calls={}, paths={}",
            semantic.file_budget, semantic.method_call_budget, semantic.path_budget
        );
        if semantic.selected_root_source_files > 0 {
            println!(
                "  analyzer selected-root files: {}/{} analyzed ({} failed, {} skipped by budget)",
                semantic.selected_root_analyzed_files,
                semantic.selected_root_source_files,
                semantic.selected_root_failed_files,
                semantic.selected_root_skipped_files
            );
        }
        println!(
            "  analyzer method calls: {}/{} queried function-resolved, {}/{} callable, {}/{} fallback ({} unresolved, {} unqueried)",
            semantic.resolved_method_calls,
            semantic.queried_method_calls,
            semantic.callable_method_calls,
            semantic.queried_method_calls,
            semantic.fallback_method_calls,
            semantic.queried_method_calls,
            semantic.unresolved_method_calls,
            semantic.unqueried_method_calls
        );
        println!(
            "  analyzer paths: {}/{} queried resolved ({} unresolved, {} unqueried)",
            semantic.resolved_paths,
            semantic.queried_paths,
            semantic.unresolved_paths,
            semantic.unqueried_paths
        );
    }
    println!(
        "production readiness: {} ({} hazard(s))",
        report.production.status,
        report.production.hazards.len()
    );
    for hazard in report
        .production
        .hazards
        .iter()
        .take(options.feedback_limit)
    {
        println!(
            "  production {} {}: {}",
            hazard.severity, hazard.code, hazard.message
        );
    }
    if report.roots.len() == 1 {
        println!("root: {}", report.root);
    } else {
        println!("roots:");
        for root in &report.roots {
            println!("  {root}");
        }
    }
    println!("files written: {}", report.files_written);
    println!(
        "generation timing: total={}ms analyzer={}ms manifest={}ms parse={}ms reduce={}ms render={}ms",
        report.timings.total_ms,
        report.timings.analyzer_ms,
        report.timings.manifest_ms,
        report.timings.parse_ms,
        report.timings.reduce_ms,
        report.timings.render_ms
    );
    println!("packages: {}", report.packages.join(", "));
    print_rendered_symbol_summary(&report, "callable", "rendered callables");
    print_rendered_symbol_summary(&report, "item", "rendered items");
    let rendered_usage_contract = rendered_usage_contract(&report);
    println!(
        "rendered usage contract: used={} blocked_by_unknown={} invalid={}",
        rendered_usage_contract.used,
        rendered_usage_contract.blocked_by_unknown,
        rendered_usage_contract.invalid.len()
    );
    write_event_log(
        &options,
        "usage_contract",
        if rendered_usage_contract.invalid.is_empty() {
            "passed"
        } else {
            "failed"
        },
        "validate rendered source only contains used or blocked_by_unknown symbols",
        "rendered prunable/unused symbols indicate redundant code escaped pruning",
        event_fields(&[
            ("used", serde_json::json!(rendered_usage_contract.used)),
            (
                "blocked_by_unknown",
                serde_json::json!(rendered_usage_contract.blocked_by_unknown),
            ),
            (
                "invalid",
                serde_json::json!(rendered_usage_contract.invalid),
            ),
        ]),
    )?;
    if let Some(report_path) = slice_report_path(&options) {
        write_generate_report(&report, &report_path)?;
        println!("slice report: {}", report_path.display());
    }
    validation.gates.push(ValidationGateReport {
        name: "generation".to_string(),
        status: "passed".to_string(),
        reason: "slice workspace was generated".to_string(),
        report_path: slice_report_path(&options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    if rendered_usage_contract.invalid.is_empty() {
        validation.gates.push(ValidationGateReport {
            name: "rendered_usage_contract".to_string(),
            status: "passed".to_string(),
            reason: format!(
                "rendered source contains {} used and {} blocked_by_unknown symbols",
                rendered_usage_contract.used, rendered_usage_contract.blocked_by_unknown
            ),
            report_path: slice_report_path(&options),
            error_count: Some(0),
            warning_count: Some(rendered_usage_contract.blocked_by_unknown),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
    } else {
        let reason = format!(
            "rendered source contains invalid usage decisions: {}",
            rendered_usage_contract.invalid_preview()
        );
        validation.gates.push(ValidationGateReport {
            name: "rendered_usage_contract".to_string(),
            status: "failed".to_string(),
            reason: reason.clone(),
            report_path: slice_report_path(&options),
            error_count: Some(rendered_usage_contract.invalid.len()),
            warning_count: Some(rendered_usage_contract.blocked_by_unknown),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    if let Some(reason) = record_semantic_proof_gate(&options, &mut validation, &report) {
        write_event_log(
            &options,
            "semantic_proof",
            "failed",
            "require RA semantic proof for selected roots",
            &reason,
            event_fields(&[]),
        )?;
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    record_production_readiness_gate(
        &options,
        &mut validation,
        &report.production,
        "before compiler feedback",
    );
    write_event_log(
        &options,
        "production_readiness",
        &report.production.status,
        "classify remaining unknown surfaces before compiler feedback",
        "warning hazards may continue to compiler feedback; error hazards fail closed",
        event_fields(&[
            (
                "hazards",
                serde_json::json!(report.production.hazards.len()),
            ),
            (
                "hazard_codes",
                serde_json::json!(report
                    .production
                    .hazards
                    .iter()
                    .map(|hazard| hazard.code.clone())
                    .collect::<Vec<_>>()),
            ),
        ]),
    )?;
    if production_readiness_blocks_validation(&options, &report.production) {
        let reason =
            "production readiness reported error hazards before compiler feedback".to_string();
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    let uncovered_targets =
        uncovered_validation_targets(&report.targets, &options.cargo_check_args);
    if !uncovered_targets.is_empty() {
        let reason = format!(
            "selected target(s) are not covered by cargo check args: {}",
            uncovered_targets.join(", ")
        );
        write_event_log(
            &options,
            "target_coverage",
            "failed",
            "verify cargo check arguments cover selected targets",
            &reason,
            event_fields(&[("uncovered_targets", serde_json::json!(uncovered_targets))]),
        )?;
        validation.gates.push(ValidationGateReport {
            name: "target_coverage".to_string(),
            status: "failed".to_string(),
            reason: reason.clone(),
            report_path: slice_report_path(&options),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    validation.gates.push(ValidationGateReport {
        name: "target_coverage".to_string(),
        status: "passed".to_string(),
        reason: "selected targets are covered by cargo check arguments".to_string(),
        report_path: slice_report_path(&options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    write_event_log(
        &options,
        "target_coverage",
        "passed",
        "verify cargo check arguments cover selected targets",
        "selected targets are covered by the requested cargo check scope",
        event_fields(&[("targets", serde_json::json!(report.targets.len()))]),
    )?;
    if let Err(error) = refresh_generated_lockfile_for_locked_validation(&options, &mut validation)
    {
        let reason = error.to_string();
        write_event_log(
            &options,
            "lockfile",
            "failed",
            "reconcile generated lockfile before validation",
            &reason,
            event_fields(&[]),
        )?;
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }
    if let Some(report) = &baseline {
        write_baseline_report(&options, report)?;
        println!(
            "baseline report: {}",
            baseline_report_path(&options).display()
        );
    }

    if options.run_preflight
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0
    {
        write_event_log(
            &options,
            "preflight",
            "started",
            "run fast structural validation before cargo feedback",
            "preflight catches malformed generated workspaces before slower compiler checks",
            event_fields(&[(
                "report_path",
                serde_json::json!(preflight_report_path(&options)),
            )]),
        )?;
        let preflight = run_preflight(&options)?;
        write_event_log(
            &options,
            "preflight",
            if preflight.success {
                "passed"
            } else {
                "failed"
            },
            "validate generated workspace structure",
            if preflight.success {
                "generated workspace passed fast structural validation"
            } else {
                "generated workspace failed fast structural validation"
            },
            event_fields(&[
                ("packages", serde_json::json!(preflight.packages)),
                ("rust_files", serde_json::json!(preflight.rust_files)),
                ("errors", serde_json::json!(preflight.error_count())),
                ("warnings", serde_json::json!(preflight.warning_count())),
            ]),
        )?;
        validation.gates.push(ValidationGateReport {
            name: "preflight".to_string(),
            status: if preflight.success {
                "passed"
            } else {
                "failed"
            }
            .to_string(),
            reason: if preflight.success {
                "generated workspace passed fast structural validation".to_string()
            } else {
                "generated workspace failed fast structural validation".to_string()
            },
            report_path: Some(preflight_report_path(&options)),
            error_count: Some(preflight.error_count()),
            warning_count: Some(preflight.warning_count()),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        if !preflight.success {
            finish_validation_with_decision_log(
                &options,
                &mut validation,
                &report,
                "rejected",
                Some("generated workspace failed fast preflight validation"),
            )?;
            return Err("generated workspace failed fast preflight validation".into());
        }
    }

    if options.feedback_repair_iterations > 0 {
        write_event_log(
            &options,
            "feedback_repair",
            "started",
            "run compiler feedback with conservative repair",
            "compiler feedback validates macro expansion and unresolved semantic surfaces",
            event_fields(&[
                (
                    "iterations",
                    serde_json::json!(options.feedback_repair_iterations),
                ),
                (
                    "target_dir",
                    serde_json::json!(feedback_target_dir(&options)),
                ),
            ]),
        )?;
        if let Err(error) = run_feedback_repair_loop(&options, baseline.as_ref(), &mut validation) {
            let reason = error.to_string();
            finish_validation_with_decision_log(
                &options,
                &mut validation,
                &report,
                "rejected",
                Some(&reason),
            )?;
            return Err(reason.into());
        }
    } else if options.feedback_iterations > 0 {
        write_event_log(
            &options,
            "feedback",
            "started",
            "run compiler feedback",
            "compiler feedback validates the generated slice after top-down pruning",
            event_fields(&[
                ("iterations", serde_json::json!(options.feedback_iterations)),
                (
                    "target_dir",
                    serde_json::json!(feedback_target_dir(&options)),
                ),
            ]),
        )?;
        if let Err(error) = run_feedback_loop(&options, baseline.as_ref(), &mut validation) {
            let reason = error.to_string();
            finish_validation_with_decision_log(
                &options,
                &mut validation,
                &report,
                "rejected",
                Some(&reason),
            )?;
            return Err(reason.into());
        }
    } else if options.run_check {
        write_event_log(
            &options,
            "check",
            "started",
            "run plain generated workspace cargo check",
            "plain check validates generated buildability without feedback widening",
            event_fields(&[(
                "target_dir",
                serde_json::json!(feedback_target_dir(&options)),
            )]),
        )?;
        if let Err(error) = run_plain_check_gate(&options, &mut validation) {
            let reason = error.to_string();
            write_decision_log(
                &options,
                &report,
                Some(&validation),
                "rejected",
                Some(&reason),
            )?;
            return Err(error);
        }
    }

    if let Err(error) =
        run_production_validation_matrix(&options, &report.production, &mut validation)
    {
        let reason = error.to_string();
        finish_validation_with_decision_log(
            &options,
            &mut validation,
            &report,
            "rejected",
            Some(&reason),
        )?;
        return Err(reason.into());
    }

    record_final_production_readiness(&options, &mut validation);
    write_event_log(
        &options,
        "final",
        "accepted",
        "accept generated slice after all requested validation gates",
        "no validation gate rejected the generated slice",
        event_fields(&[("gates", serde_json::json!(validation.gates.len()))]),
    )?;
    finish_validation_with_decision_log(&options, &mut validation, &report, "accepted", None)?;
    Ok(())
}

fn print_rendered_symbol_summary(report: &GenerateReport, kind: &str, label: &str) {
    println!("{label}:");
    let decisions = match kind {
        "callable" => Some(&report.usage.rendered_decision_map.callables),
        "item" => Some(&report.usage.rendered_decision_map.items),
        _ => None,
    };
    for entry in report
        .usage
        .rendered_symbols
        .entries
        .iter()
        .filter(|entry| entry.kind == kind)
    {
        let classification = decisions
            .and_then(|decisions| decisions.get(&entry.id))
            .unwrap_or(&entry.classification);
        println!("  {} [{}]", entry.id, classification);
    }
}

#[derive(Debug, Default)]
struct RenderedUsageContract {
    used: usize,
    blocked_by_unknown: usize,
    invalid: Vec<String>,
}

impl RenderedUsageContract {
    fn invalid_preview(&self) -> String {
        let mut preview = self.invalid.iter().take(5).cloned().collect::<Vec<_>>();
        if self.invalid.len() > preview.len() {
            preview.push(format!("... {} more", self.invalid.len() - preview.len()));
        }
        preview.join(", ")
    }

    fn summary_fields(&self) -> BTreeMap<String, serde_json::Value> {
        event_fields(&[
            ("used", serde_json::json!(self.used)),
            (
                "blocked_by_unknown",
                serde_json::json!(self.blocked_by_unknown),
            ),
            ("invalid", serde_json::json!(self.invalid.len())),
            ("invalid_preview", serde_json::json!(self.invalid_preview())),
        ])
    }
}

fn rendered_usage_contract(report: &GenerateReport) -> RenderedUsageContract {
    let mut contract = RenderedUsageContract::default();
    let decisions = &report.usage.rendered_decision_map;
    accumulate_rendered_usage_decisions("callable", &decisions.callables, &mut contract);
    accumulate_rendered_usage_decisions("item", &decisions.items, &mut contract);
    accumulate_rendered_usage_decisions("member", &decisions.members, &mut contract);
    accumulate_rendered_usage_decisions("assoc_item", &decisions.assoc_items, &mut contract);
    accumulate_rendered_usage_decisions(
        "trait_default_method",
        &decisions.trait_default_methods,
        &mut contract,
    );
    contract
}

fn decision_log_decision_samples(
    kind: &str,
    decisions: &BTreeMap<String, String>,
    limit: usize,
) -> Vec<String> {
    let mut samples = Vec::new();
    for preferred in [
        "prunable",
        "unclassified",
        "blocked_by_unknown",
        "used",
        "retained",
    ] {
        for (id, decision) in decisions {
            if decision != preferred {
                continue;
            }
            samples.push(format!("{kind} {id} [{decision}]"));
            if samples.len() >= limit {
                return samples;
            }
        }
    }
    for (id, decision) in decisions {
        let sample = format!("{kind} {id} [{decision}]");
        if samples.contains(&sample) {
            continue;
        }
        samples.push(sample);
        if samples.len() >= limit {
            break;
        }
    }
    samples
}

fn decision_log_retained_source_file_samples(report: &GenerateReport, limit: usize) -> Vec<String> {
    let mut files: BTreeMap<PathBuf, (usize, usize)> = BTreeMap::new();
    for location in &report.source_map.callables {
        if !location.reachable {
            continue;
        }
        files
            .entry(location.span.file.clone())
            .and_modify(|counts| counts.0 += 1)
            .or_insert((1, 0));
    }
    for location in &report.source_map.items {
        if !location.reachable {
            continue;
        }
        files
            .entry(location.span.file.clone())
            .and_modify(|counts| counts.1 += 1)
            .or_insert((0, 1));
    }
    files
        .into_iter()
        .take(limit)
        .map(|(file, (callables, items))| {
            format!(
                "{} (reachable_callables={callables}, reachable_items={items})",
                file.display()
            )
        })
        .collect()
}

fn retained_source_file_count(report: &GenerateReport) -> usize {
    let mut files = BTreeSet::new();
    for location in &report.source_map.callables {
        if location.reachable {
            files.insert(location.span.file.clone());
        }
    }
    for location in &report.source_map.items {
        if location.reachable {
            files.insert(location.span.file.clone());
        }
    }
    files.len()
}

#[derive(Debug, Default)]
struct PackageSurfaceCounts {
    reachable_callables: usize,
    reachable_items: usize,
    rendered_callables: usize,
    rendered_items: usize,
    rendered_members: usize,
    rendered_assoc_items: usize,
    used_callables: usize,
    used_items: usize,
    blocked_by_unknown_callables: usize,
    blocked_by_unknown_items: usize,
    prunable_callables: usize,
    prunable_items: usize,
    unknown_surfaces: usize,
    macro_surfaces: usize,
}

fn decision_log_package_surface_audit(
    report: &GenerateReport,
) -> BTreeMap<String, PackageSurfaceCounts> {
    let mut audit: BTreeMap<String, PackageSurfaceCounts> = BTreeMap::new();
    for package in &report.packages {
        audit.entry(package.clone()).or_default();
    }
    for root in &report.roots {
        audit.entry(root.package().to_string()).or_default();
    }
    for callable in &report.reachable {
        audit
            .entry(callable.package().to_string())
            .or_default()
            .reachable_callables += 1;
    }
    for item in &report.reachable_items {
        audit
            .entry(item.package.clone())
            .or_default()
            .reachable_items += 1;
    }
    for callable in &report.usage.used.callables {
        audit
            .entry(callable.package().to_string())
            .or_default()
            .used_callables += 1;
    }
    for item in &report.usage.used.items {
        audit.entry(item.package.clone()).or_default().used_items += 1;
    }
    for callable in &report.usage.blocked_by_unknown.callables {
        audit
            .entry(callable.package().to_string())
            .or_default()
            .blocked_by_unknown_callables += 1;
    }
    for item in &report.usage.blocked_by_unknown.items {
        audit
            .entry(item.package.clone())
            .or_default()
            .blocked_by_unknown_items += 1;
    }
    for callable in &report.usage.prunable.callables {
        audit
            .entry(callable.package().to_string())
            .or_default()
            .prunable_callables += 1;
    }
    for item in &report.usage.prunable.items {
        audit
            .entry(item.package.clone())
            .or_default()
            .prunable_items += 1;
    }
    count_rendered_decisions_by_package(
        &mut audit,
        &report.usage.rendered_decision_map.callables,
        |counts| counts.rendered_callables += 1,
    );
    count_rendered_decisions_by_package(
        &mut audit,
        &report.usage.rendered_decision_map.items,
        |counts| counts.rendered_items += 1,
    );
    count_rendered_decisions_by_package(
        &mut audit,
        &report.usage.rendered_decision_map.members,
        |counts| counts.rendered_members += 1,
    );
    count_rendered_decisions_by_package(
        &mut audit,
        &report.usage.rendered_decision_map.assoc_items,
        |counts| counts.rendered_assoc_items += 1,
    );
    for surface in &report.usage.unknown {
        let package = surface
            .details
            .iter()
            .find_map(|detail| detail.package.clone())
            .unwrap_or_else(|| "<unknown-package>".to_string());
        audit.entry(package).or_default().unknown_surfaces += 1;
    }
    for surface in &report.macro_surfaces.surfaces {
        audit
            .entry(surface.package.clone())
            .or_default()
            .macro_surfaces += 1;
    }
    audit
}

fn count_rendered_decisions_by_package(
    audit: &mut BTreeMap<String, PackageSurfaceCounts>,
    decisions: &BTreeMap<String, String>,
    mut count: impl FnMut(&mut PackageSurfaceCounts),
) {
    for id in decisions.keys() {
        count(audit.entry(symbol_package(id)).or_default());
    }
}

fn symbol_package(id: &str) -> String {
    id.split("::").next().unwrap_or(id).to_string()
}

fn decision_log_package_surface_samples(
    audit: &BTreeMap<String, PackageSurfaceCounts>,
    root_packages: &BTreeSet<String>,
    limit: usize,
) -> Vec<String> {
    audit
        .iter()
        .take(limit)
        .map(|(package, counts)| {
            let role = if root_packages.contains(package) {
                "root"
            } else {
                "downstream_dependency"
            };
            format!(
                "package {package} [{role}] reachable(callables={}, items={}) rendered(callables={}, items={}, members={}, assoc_items={}) used(callables={}, items={}) blocked_unknown(callables={}, items={}, surfaces={}) prunable(callables={}, items={}) macro_surfaces={}",
                counts.reachable_callables,
                counts.reachable_items,
                counts.rendered_callables,
                counts.rendered_items,
                counts.rendered_members,
                counts.rendered_assoc_items,
                counts.used_callables,
                counts.used_items,
                counts.blocked_by_unknown_callables,
                counts.blocked_by_unknown_items,
                counts.unknown_surfaces,
                counts.prunable_callables,
                counts.prunable_items,
                counts.macro_surfaces,
            )
        })
        .collect()
}

fn decision_log_usage_samples(
    report: &GenerateReport,
    rendered_usage: &RenderedUsageContract,
    limit: usize,
) -> Vec<String> {
    let mut evidence = rendered_usage
        .invalid
        .iter()
        .take(limit)
        .map(|invalid| format!("invalid rendered decision {invalid}"))
        .collect::<Vec<_>>();
    push_usage_samples(
        &mut evidence,
        limit,
        "used callable",
        report.usage.used.callables.iter().map(ToString::to_string),
    );
    push_usage_samples(
        &mut evidence,
        limit,
        "used item",
        report.usage.used.items.iter().map(ToString::to_string),
    );
    push_usage_samples(
        &mut evidence,
        limit,
        "blocked_by_unknown callable",
        report
            .usage
            .blocked_by_unknown
            .callables
            .iter()
            .map(ToString::to_string),
    );
    push_usage_samples(
        &mut evidence,
        limit,
        "blocked_by_unknown item",
        report
            .usage
            .blocked_by_unknown
            .items
            .iter()
            .map(ToString::to_string),
    );
    push_usage_samples(
        &mut evidence,
        limit,
        "pruned callable",
        report
            .usage
            .prunable
            .callables
            .iter()
            .map(ToString::to_string),
    );
    push_usage_samples(
        &mut evidence,
        limit,
        "pruned item",
        report.usage.prunable.items.iter().map(ToString::to_string),
    );
    evidence
}

fn decision_log_semantic_usage_proof_evidence(
    report: &GenerateReport,
    limit: usize,
) -> Vec<String> {
    let proof = &report.usage.semantic_proof;
    let summary = &proof.summary;
    let mut evidence = Vec::new();
    if summary.cfg_inactive_callables + summary.cfg_inactive_items > 0 {
        evidence.push(format!(
            "cfg_inactive_discharge callables={} items={} reason=current target cfg excludes these prunable candidates",
            summary.cfg_inactive_callables, summary.cfg_inactive_items
        ));
    }
    if summary.source_file_pruned_callables + summary.source_file_pruned_items > 0 {
        evidence.push(format!(
            "source_file_pruned_discharge callables={} items={} reason=source files containing only prunable candidates were not rendered",
            summary.source_file_pruned_callables, summary.source_file_pruned_items
        ));
    }
    if summary.structural_pruned_items > 0 {
        evidence.push(format!(
            "structural_pruned_discharge items={} reason=module declarations whose subtrees contain no retained symbols were not rendered",
            summary.structural_pruned_items
        ));
    }
    push_usage_samples(
        &mut evidence,
        limit,
        "unproven semantic callable",
        proof.unproven.callables.iter().map(ToString::to_string),
    );
    push_usage_samples(
        &mut evidence,
        limit,
        "unproven semantic item",
        proof.unproven.items.iter().map(ToString::to_string),
    );
    if evidence.is_empty() {
        evidence.push("all retained-package prunable candidates are semantically proven or explicitly discharged".to_string());
    }
    evidence.truncate(limit);
    evidence
}

fn push_usage_samples(
    evidence: &mut Vec<String>,
    limit: usize,
    label: &str,
    ids: impl Iterator<Item = String>,
) {
    for id in ids {
        if evidence.len() >= limit {
            break;
        }
        evidence.push(format!("{label} {id}"));
    }
}

fn decision_log_feedback_diagnostics(
    options: &CliOptions,
    validation: &ValidationReport,
    limit: usize,
) -> FeedbackDiagnosticLogSummary {
    let mut summary = FeedbackDiagnosticLogSummary::default();
    summary.feedback_baseline_compared = validation.feedback_baseline_compared;
    summary.feedback_baseline_known_errors = validation.feedback_baseline_known_errors;
    summary.feedback_baseline_new_errors = validation.feedback_baseline_new_errors;
    if let Some(compared) = summary.feedback_baseline_compared {
        if summary.evidence.len() < limit {
            summary.evidence.push(format!(
                "source baseline compared={} known_errors={} new_errors={}",
                compared,
                summary.feedback_baseline_known_errors.unwrap_or(0),
                summary.feedback_baseline_new_errors.unwrap_or(0)
            ));
        }
    }
    for report_path in decision_log_feedback_report_paths(validation) {
        match read_feedback_report_for_decision_log(&report_path) {
            Ok(report) => {
                accumulate_feedback_diagnostic_summary(
                    options,
                    &report,
                    Some(&report_path),
                    &mut summary,
                    limit,
                );
            }
            Err(error) => {
                summary.unreadable_reports += 1;
                if summary.evidence.len() < limit {
                    summary.evidence.push(format!(
                        "unreadable report {}: {}",
                        report_path.display(),
                        error
                    ));
                }
            }
        }
    }
    summary
}

fn feedback_diagnostic_summary_for_check(
    options: &CliOptions,
    report: &CheckReport,
    limit: usize,
) -> FeedbackDiagnosticLogSummary {
    let mut summary = FeedbackDiagnosticLogSummary::default();
    accumulate_feedback_diagnostic_summary(options, report, None, &mut summary, limit);
    summary
}

fn accumulate_feedback_diagnostic_summary(
    options: &CliOptions,
    report: &CheckReport,
    report_path: Option<&Path>,
    summary: &mut FeedbackDiagnosticLogSummary,
    limit: usize,
) {
    summary.reports += 1;
    summary.diagnostics += report.diagnostics.len();
    summary.errors += report.error_count();
    summary.warnings += report.warning_count();
    summary.widening_candidates += report.widening.candidates.len();
    summary.widening_hazards += report.widening.hazards.len();
    if report.error_count() > 0 {
        match resolve_feedback_widening_roots(
            &options.workspace_root,
            &report.diagnostics,
            &options.root_selectors,
        ) {
            Ok(resolution) => {
                summary.resolution_reports += 1;
                summary.resolution_matched_roots += resolution.matched_roots.len();
                summary.resolution_skipped_no_match += resolution.skipped_no_match;
                summary.resolution_skipped_too_many_matches += resolution.skipped_too_many_matches;
                summary.glob_import_context_entries += resolution
                    .entries
                    .iter()
                    .filter(|entry| {
                        !entry.glob_imports.is_empty()
                            || !entry.glob_import_module_roots.is_empty()
                            || !entry.glob_import_provider_internal_globs.is_empty()
                    })
                    .count();
                summary.source_api_mismatch_candidates +=
                    decision_log_source_api_mismatch_candidates(
                        report,
                        &resolution,
                        &mut summary.evidence,
                        limit,
                    );
                summary.glob_import_private_upstream_candidates +=
                    decision_log_glob_import_private_upstream_candidates(
                        &resolution,
                        &mut summary.evidence,
                        limit,
                    );
                summary.glob_import_missing_export_candidates +=
                    decision_log_glob_import_missing_export_candidates(
                        &resolution,
                        &mut summary.evidence,
                        limit,
                    );
                decision_log_feedback_resolution_evidence(
                    &resolution,
                    &mut summary.evidence,
                    limit,
                );
            }
            Err(error) => {
                summary.resolution_errors += 1;
                if summary.evidence.len() < limit {
                    let report_label = report_path
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "<in-memory>".to_string());
                    summary.evidence.push(format!(
                        "feedback resolution unreadable for {report_label}: {error}",
                    ));
                }
            }
        }
    }
    if summary.evidence.len() < limit {
        let report_label = report_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<in-memory>".to_string());
        summary.evidence.push(format!(
            "report {} success={} errors={} warnings={} duration_ms={}",
            report_label,
            report.success,
            report.error_count(),
            report.warning_count(),
            report.duration_ms
        ));
    }
    for candidate in &report.widening.candidates {
        if summary.evidence.len() >= limit {
            break;
        }
        let symbol = candidate.symbol.as_deref().unwrap_or("<unknown>");
        let location = candidate
            .file_name
            .as_deref()
            .zip(candidate.line_start)
            .map(|(file, line)| format!("{file}:{line}"))
            .unwrap_or_else(|| "<unknown>".to_string());
        summary.evidence.push(format!(
            "widening {} `{}` at {}: {}",
            candidate.kind, symbol, location, candidate.action
        ));
    }
    for diagnostic in prioritized_diagnostics(&report.diagnostics) {
        if summary.evidence.len() >= limit {
            break;
        }
        summary
            .evidence
            .push(decision_log_diagnostic_summary(diagnostic));
    }
}

fn feedback_diagnostic_status(summary: &FeedbackDiagnosticLogSummary) -> String {
    if summary.errors > 0 {
        "errors".to_string()
    } else if summary.warnings > 0 {
        "warnings".to_string()
    } else if summary.unreadable_reports > 0 {
        "partial".to_string()
    } else {
        "clean".to_string()
    }
}

fn feedback_diagnostic_metrics(
    summary: &FeedbackDiagnosticLogSummary,
) -> BTreeMap<String, serde_json::Value> {
    let mut metrics = BTreeMap::new();
    metrics.insert("reports".to_string(), serde_json::json!(summary.reports));
    metrics.insert(
        "unreadable_reports".to_string(),
        serde_json::json!(summary.unreadable_reports),
    );
    metrics.insert(
        "diagnostics".to_string(),
        serde_json::json!(summary.diagnostics),
    );
    metrics.insert("errors".to_string(), serde_json::json!(summary.errors));
    metrics.insert("warnings".to_string(), serde_json::json!(summary.warnings));
    metrics.insert(
        "widening_candidates".to_string(),
        serde_json::json!(summary.widening_candidates),
    );
    metrics.insert(
        "widening_hazards".to_string(),
        serde_json::json!(summary.widening_hazards),
    );
    metrics.insert(
        "resolution_reports".to_string(),
        serde_json::json!(summary.resolution_reports),
    );
    metrics.insert(
        "resolution_errors".to_string(),
        serde_json::json!(summary.resolution_errors),
    );
    metrics.insert(
        "resolution_matched_roots".to_string(),
        serde_json::json!(summary.resolution_matched_roots),
    );
    metrics.insert(
        "resolution_skipped_no_match".to_string(),
        serde_json::json!(summary.resolution_skipped_no_match),
    );
    metrics.insert(
        "resolution_skipped_too_many_matches".to_string(),
        serde_json::json!(summary.resolution_skipped_too_many_matches),
    );
    metrics.insert(
        "source_api_mismatch_candidates".to_string(),
        serde_json::json!(summary.source_api_mismatch_candidates),
    );
    metrics.insert(
        "glob_import_context_entries".to_string(),
        serde_json::json!(summary.glob_import_context_entries),
    );
    metrics.insert(
        "glob_import_missing_export_candidates".to_string(),
        serde_json::json!(summary.glob_import_missing_export_candidates),
    );
    metrics.insert(
        "glob_import_private_upstream_candidates".to_string(),
        serde_json::json!(summary.glob_import_private_upstream_candidates),
    );
    if let Some(compared) = summary.feedback_baseline_compared {
        metrics.insert(
            "feedback_baseline_compared".to_string(),
            serde_json::json!(compared),
        );
    }
    if let Some(known_errors) = summary.feedback_baseline_known_errors {
        metrics.insert(
            "feedback_baseline_known_errors".to_string(),
            serde_json::json!(known_errors),
        );
    }
    if let Some(new_errors) = summary.feedback_baseline_new_errors {
        metrics.insert(
            "feedback_baseline_new_errors".to_string(),
            serde_json::json!(new_errors),
        );
    }
    metrics
}

fn decision_log_feedback_resolution_evidence(
    resolution: &FeedbackRootResolutionReport,
    evidence: &mut Vec<String>,
    limit: usize,
) {
    if evidence.len() < limit {
        evidence.push(format!(
			"feedback resolution matched_roots={} skipped_no_match={} skipped_too_many_matches={} skipped_marked_roots={} candidate_symbols={}",
			resolution.matched_roots.len(),
			resolution.skipped_no_match,
			resolution.skipped_too_many_matches,
			resolution.skipped_marked_roots,
			resolution.candidate_symbols
		));
    }
    for entry in &resolution.entries {
        if evidence.len() >= limit {
            break;
        }
        let package_hint = entry.package_hint.as_deref().unwrap_or("<none>");
        let diagnostic_file = entry
            .diagnostic_file
            .as_deref()
            .map(|file| format!(" diagnostic_file={file}"))
            .unwrap_or_default();
        let glob_imports = (!entry.glob_imports.is_empty())
            .then(|| format!(" glob_imports={}", entry.glob_imports.join(",")))
            .unwrap_or_default();
        let glob_import_roots = (!entry.glob_import_module_roots.is_empty())
            .then(|| {
                format!(
                    " glob_import_module_roots={}",
                    entry.glob_import_module_roots.join(",")
                )
            })
            .unwrap_or_default();
        let provider_internal_globs = (!entry.glob_import_provider_internal_globs.is_empty())
            .then(|| {
                format!(
                    " provider_internal_globs={}",
                    entry.glob_import_provider_internal_globs.join(",")
                )
            })
            .unwrap_or_default();
        evidence.push(format!(
				"resolution {} symbol=`{}` package_hint={} matches={} retained_roots={} skipped_marked_roots={} action={} roots={}{}{}{}{}",
				entry.code,
				entry.symbol,
				package_hint,
				entry.matches,
				entry.retained_roots,
			entry.skipped_marked_roots,
			entry.action,
				entry.roots.join(","),
				diagnostic_file,
				glob_imports,
				glob_import_roots,
				provider_internal_globs
			));
    }
    if resolution.entries_truncated && evidence.len() < limit {
        evidence.push("feedback resolution entries truncated".to_string());
    }
}

fn decision_log_source_api_mismatch_candidates(
    report: &CheckReport,
    resolution: &FeedbackRootResolutionReport,
    evidence: &mut Vec<String>,
    limit: usize,
) -> usize {
    let mut candidates = 0;
    for diagnostic in &report.diagnostics {
        if !matches!(diagnostic.code.as_deref(), Some("E0432" | "E0433")) {
            continue;
        }
        let suggestions = decision_log_suggestion_replacements(diagnostic);
        if suggestions.is_empty() {
            continue;
        }
        let suggestion_matches = suggestions
            .iter()
            .filter(|replacement| {
                resolution.entries.iter().any(|entry| {
                    entry.name.as_deref() == Some(replacement.as_str()) && entry.matches > 0
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        if suggestion_matches.is_empty() {
            continue;
        }
        for symbol in decision_log_backticked_symbols(&diagnostic.message)
            .into_iter()
            .chain(
                diagnostic
                    .rendered
                    .as_deref()
                    .into_iter()
                    .flat_map(decision_log_backticked_symbols),
            )
        {
            if !symbol.contains("::") {
                continue;
            }
            let exact_missing = resolution
                .entries
                .iter()
                .any(|entry| entry.symbol == symbol && entry.matches == 0);
            if !exact_missing {
                continue;
            }
            candidates += 1;
            if evidence.len() < limit {
                evidence.push(format!(
					"source_api_mismatch_candidate unresolved_path=`{}` exact_matches=0 suggested_symbol_matches={} suggestions={}",
					symbol,
					suggestion_matches.join(","),
					suggestions.join(",")
				));
            }
            break;
        }
    }
    candidates
}

fn decision_log_glob_import_missing_export_candidates(
    resolution: &FeedbackRootResolutionReport,
    evidence: &mut Vec<String>,
    limit: usize,
) -> usize {
    let mut candidates = 0;
    for entry in &resolution.entries {
        if entry.matches != 0
            || entry.glob_import_module_roots.is_empty()
            || is_glob_import_private_upstream_candidate(entry)
        {
            continue;
        }
        candidates += 1;
        if evidence.len() < limit {
            let diagnostic_file = entry.diagnostic_file.as_deref().unwrap_or("<unknown>");
            let provider_internal_globs = if entry.glob_import_provider_internal_globs.is_empty() {
                "<none>".to_string()
            } else {
                entry.glob_import_provider_internal_globs.join(",")
            };
            evidence.push(format!(
					"glob_import_missing_export_candidate symbol=`{}` diagnostic_file={} glob_imports={} provider_module_roots={} provider_internal_globs={} reason=glob provider module resolved but no project-local export/root matched the missing symbol",
					entry.symbol,
					diagnostic_file,
					entry.glob_imports.join(","),
					entry.glob_import_module_roots.join(","),
					provider_internal_globs
				));
        }
    }
    candidates
}

fn decision_log_glob_import_private_upstream_candidates(
    resolution: &FeedbackRootResolutionReport,
    evidence: &mut Vec<String>,
    limit: usize,
) -> usize {
    let mut candidates = 0;
    for entry in &resolution.entries {
        if !is_glob_import_private_upstream_candidate(entry) {
            continue;
        }
        candidates += 1;
        if evidence.len() < limit {
            let diagnostic_file = entry.diagnostic_file.as_deref().unwrap_or("<unknown>");
            evidence.push(format!(
				"glob_import_private_upstream_candidate symbol=`{}` diagnostic_file={} glob_imports={} provider_module_roots={} provider_internal_globs={} reason=glob provider module exists but only imports upstream glob names privately; missing symbol is not a public export of the imported module",
				entry.symbol,
				diagnostic_file,
				entry.glob_imports.join(","),
				entry.glob_import_module_roots.join(","),
				entry.glob_import_provider_internal_globs.join(",")
			));
        }
    }
    candidates
}

fn is_glob_import_private_upstream_candidate(entry: &FeedbackRootResolutionEntry) -> bool {
    entry.matches == 0
        && !entry.glob_import_module_roots.is_empty()
        && !entry.glob_import_provider_internal_globs.is_empty()
}

fn decision_log_suggestion_replacements(diagnostic: &CheckDiagnostic) -> Vec<String> {
    let mut replacements = diagnostic
        .suggestions
        .iter()
        .map(|suggestion| suggestion.suggested_replacement.trim().to_string())
        .filter(|replacement| !replacement.is_empty())
        .collect::<Vec<_>>();
    if replacements.is_empty() {
        if let Some(rendered) = &diagnostic.rendered {
            replacements.extend(
                decision_log_backticked_symbols(rendered)
                    .into_iter()
                    .filter(|symbol| !symbol.contains("::")),
            );
        }
    }
    replacements.sort();
    replacements.dedup();
    replacements
}

fn decision_log_backticked_symbols(text: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut remaining = text;
    while let Some((_, tail)) = remaining.split_once('`') {
        let Some((symbol, after)) = tail.split_once('`') else {
            break;
        };
        if !symbol.is_empty() {
            symbols.push(symbol.to_string());
        }
        remaining = after;
    }
    symbols
}

fn decision_log_feedback_report_paths(validation: &ValidationReport) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen_reports = BTreeSet::new();
    for attempt in &validation.attempts {
        if seen_reports.insert(attempt.report_path.clone()) {
            paths.push(attempt.report_path.clone());
        }
    }
    for gate in &validation.gates {
        let Some(report_path) = &gate.report_path else {
            continue;
        };
        if !decision_log_gate_has_feedback_report(gate, report_path) {
            continue;
        }
        if seen_reports.insert(report_path.clone()) {
            paths.push(report_path.clone());
        }
    }
    paths
}

fn decision_log_gate_has_feedback_report(gate: &ValidationGateReport, report_path: &Path) -> bool {
    if matches!(gate.name.as_str(), "feedback" | "feedback-repair") {
        return true;
    }
    report_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains("feedback"))
}

fn read_feedback_report_for_decision_log(
    path: &Path,
) -> Result<CheckReport, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let report = serde_json::from_slice(&bytes)?;
    Ok(report)
}

fn decision_log_diagnostic_summary(diagnostic: &CheckDiagnostic) -> String {
    let code = diagnostic.code.as_deref().unwrap_or("no_code");
    let mut primary_spans = diagnostic
        .spans
        .iter()
        .filter(|span| span.is_primary)
        .take(2)
        .map(|span| {
            format!(
                "{}:{}:{}-{}",
                span.file_name, span.line_start, span.column_start, span.column_end
            )
        })
        .collect::<Vec<_>>();
    if primary_spans.is_empty() {
        primary_spans.push("<no_primary_span>".to_string());
    }
    format!(
        "{}[{}] at {}: {}",
        diagnostic.level,
        code,
        primary_spans.join(", "),
        diagnostic.message
    )
}

fn accumulate_rendered_usage_decisions(
    kind: &str,
    decisions: &BTreeMap<String, String>,
    contract: &mut RenderedUsageContract,
) {
    for (id, decision) in decisions {
        match decision.as_str() {
            "used" => contract.used += 1,
            "blocked_by_unknown" => contract.blocked_by_unknown += 1,
            _ => contract.invalid.push(format!("{kind} {id} [{decision}]")),
        }
    }
}

#[derive(Clone)]
struct CliOptions {
    analyzer_mode: AnalyzerMode,
    run_check: bool,
    feedback_iterations: usize,
    feedback_repair_iterations: usize,
    feedback_limit: usize,
    feedback_report: Option<PathBuf>,
    feedback_target_dir: Option<PathBuf>,
    feedback_timeout: Option<Duration>,
    cargo_check_args: Vec<String>,
    deny_warnings: bool,
    repair_report: Option<PathBuf>,
    run_baseline_check: bool,
    allow_baseline_failures: bool,
    baseline_report: Option<PathBuf>,
    baseline_target_dir: Option<PathBuf>,
    slice_report: Option<PathBuf>,
    decision_log: Option<PathBuf>,
    event_log: Option<PathBuf>,
    validation_report: Option<PathBuf>,
    run_preflight: bool,
    preflight_report: Option<PathBuf>,
    production_preset: bool,
    root_selectors: Vec<String>,
    random_roots: Option<usize>,
    random_root_packages: Vec<String>,
    random_seed: u64,
    batch_roots: bool,
    batch_report: Option<PathBuf>,
    workspace_root: PathBuf,
    output_root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationReport {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    production_preset: bool,
    workspace_root: PathBuf,
    output_root: PathBuf,
    cargo_check_args: Vec<String>,
    deny_warnings: bool,
    allow_baseline_failures: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_baseline_compared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_baseline_known_errors: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_baseline_new_errors: Option<usize>,
    gates: Vec<ValidationGateReport>,
    attempts: Vec<ValidationAttemptReport>,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationGateReport {
    name: String,
    status: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_warning_hazards: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_warning_hazards: Option<usize>,
}

#[derive(Debug, Serialize)]
struct BatchRootReport {
    root: String,
    output_root: PathBuf,
    status: String,
    files_written: Option<usize>,
    production_status: Option<String>,
    production_hazards: Option<usize>,
    rendered_usage_used: Option<usize>,
    rendered_usage_blocked_by_unknown: Option<usize>,
    rendered_usage_invalid: Option<usize>,
    usage_semantic_proof_status: Option<String>,
    usage_semantic_proof_required_callables: Option<usize>,
    usage_semantic_proof_required_items: Option<usize>,
    usage_semantic_proven_callables: Option<usize>,
    usage_semantic_proven_items: Option<usize>,
    usage_semantic_unproven_callables: Option<usize>,
    usage_semantic_unproven_items: Option<usize>,
    usage_semantic_cfg_inactive_callables: Option<usize>,
    usage_semantic_cfg_inactive_items: Option<usize>,
    usage_semantic_source_file_pruned_callables: Option<usize>,
    usage_semantic_source_file_pruned_items: Option<usize>,
    usage_semantic_structural_pruned_items: Option<usize>,
    usage_semantic_unmapped_callables: Option<usize>,
    usage_semantic_unmapped_items: Option<usize>,
    usage_semantic_failed_reference_query_callables: Option<usize>,
    usage_semantic_failed_reference_query_items: Option<usize>,
    semantic_proof_status: Option<String>,
    semantic_file_budget: Option<usize>,
    semantic_method_call_budget: Option<usize>,
    semantic_path_budget: Option<usize>,
    semantic_source_files: Option<usize>,
    semantic_analyzed_files: Option<usize>,
    semantic_failed_files: Option<usize>,
    semantic_skipped_files: Option<usize>,
    semantic_unresolved_method_calls: Option<usize>,
    semantic_unqueried_method_calls: Option<usize>,
    semantic_unresolved_paths: Option<usize>,
    semantic_unqueried_paths: Option<usize>,
    selected_root_semantic_source_files: Option<usize>,
    selected_root_semantic_analyzed_files: Option<usize>,
    selected_root_semantic_failed_files: Option<usize>,
    selected_root_semantic_skipped_files: Option<usize>,
    selected_root_unresolved_method_calls: Option<usize>,
    selected_root_unqueried_method_calls: Option<usize>,
    selected_root_unresolved_paths: Option<usize>,
    selected_root_unqueried_paths: Option<usize>,
    preflight_errors: Option<usize>,
    preflight_warnings: Option<usize>,
    check_success: Option<bool>,
    check_errors: Option<usize>,
    check_warnings: Option<usize>,
    feedback_diagnostics: Option<usize>,
    feedback_widening_candidates: Option<usize>,
    feedback_widening_hazards: Option<usize>,
    feedback_resolution_matched_roots: Option<usize>,
    feedback_resolution_skipped_no_match: Option<usize>,
    feedback_baseline_compared: Option<bool>,
    feedback_baseline_known_errors: Option<usize>,
    feedback_baseline_new_errors: Option<usize>,
    source_api_mismatch_candidates: Option<usize>,
    glob_import_context_entries: Option<usize>,
    glob_import_missing_export_candidates: Option<usize>,
    glob_import_private_upstream_candidates: Option<usize>,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_error: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ValidationAttemptReport {
    stage: String,
    attempt: usize,
    status: String,
    reason: String,
    report_path: PathBuf,
    cargo_success: bool,
    baseline_limited: bool,
    timed_out: bool,
    error_count: usize,
    warning_count: usize,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
    widening_candidates: usize,
    widening_hazards: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_widened_roots: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_total_changes: Option<usize>,
}

#[derive(Default)]
struct FeedbackWideningState {
    diagnostics: Vec<CheckDiagnostic>,
    seen_root_sets: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct BaselineErrorComparison {
    compared: bool,
    known_errors: usize,
    new_errors: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProductionMatrixEntry {
    name: String,
    reason: String,
    cargo_args: Vec<String>,
}

impl ValidationReport {
    fn new(options: &CliOptions) -> Self {
        Self {
            status: "running".to_string(),
            reason: None,
            production_preset: options.production_preset,
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
            cargo_check_args: options.cargo_check_args.clone(),
            deny_warnings: options.deny_warnings,
            allow_baseline_failures: options.allow_baseline_failures,
            feedback_baseline_compared: None,
            feedback_baseline_known_errors: None,
            feedback_baseline_new_errors: None,
            gates: Vec::new(),
            attempts: Vec::new(),
        }
    }

    fn add_check_gate(
        &mut self,
        name: &str,
        status: &str,
        reason: &str,
        report: &CheckReport,
        report_path: Option<PathBuf>,
        semantic_warning_hazards: usize,
    ) {
        self.gates.push(ValidationGateReport {
            name: name.to_string(),
            status: status.to_string(),
            reason: reason.to_string(),
            report_path,
            error_count: Some(report.error_count()),
            warning_count: Some(report.warning_count()),
            semantic_warning_hazards: Some(semantic_warning_hazards),
            review_warning_hazards: None,
        });
    }
}

fn parse_args() -> Result<CliOptions, Box<dyn std::error::Error>> {
    parse_args_from(std::env::args_os().skip(1))
}

fn parse_args_from<I>(args: I) -> Result<CliOptions, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = OsString>,
{
    let mut analyzer_mode = AnalyzerMode::default_for_build();
    let mut analyzer_mode_explicit = false;
    let mut run_check = false;
    let mut feedback_iterations = 0;
    let mut feedback_repair_iterations = 0;
    let mut feedback_limit = 12;
    let mut feedback_report = None;
    let mut feedback_target_dir = None;
    let mut feedback_timeout = Some(Duration::from_secs(600));
    let mut cargo_check_args = Vec::new();
    let mut deny_warnings = false;
    let mut repair_report = None;
    let mut run_baseline_check = false;
    let mut allow_baseline_failures = false;
    let mut baseline_report = None;
    let mut baseline_target_dir = None;
    let mut slice_report = None;
    let mut decision_log = None;
    let mut event_log = None;
    let mut validation_report = None;
    let mut run_preflight = false;
    let mut preflight_report = None;
    let mut production_preset = false;
    let mut root_selectors = Vec::new();
    let mut random_roots = None;
    let mut random_root_packages = Vec::new();
    let mut random_seed = 0;
    let mut batch_roots = false;
    let mut batch_report = None;
    let mut workspace_root_arg = None;
    let mut output_root_arg = None;
    let mut positional = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == OsStr::new("--check") {
            run_check = true;
        } else if arg == OsStr::new("--production") {
            production_preset = true;
            run_baseline_check = true;
            run_preflight = true;
            feedback_repair_iterations = feedback_repair_iterations.max(3);
            deny_warnings = true;
        } else if arg == OsStr::new("--analyzer") {
            let value = args
                .next()
                .ok_or("--analyzer requires a following value: syn, ra-hir, ra-feedback, or ra-hir-proc-macros")?;
            let value = value
                .to_str()
                .ok_or("--analyzer value must be valid UTF-8")?;
            analyzer_mode = value.parse::<AnalyzerMode>()?;
            analyzer_mode_explicit = true;
        } else if arg == OsStr::new("--feedback") {
            feedback_iterations = feedback_iterations.max(1);
        } else if arg == OsStr::new("--preflight") {
            run_preflight = true;
        } else if arg == OsStr::new("--baseline-check") {
            run_baseline_check = true;
        } else if arg == OsStr::new("--allow-baseline-failures") {
            allow_baseline_failures = true;
            run_baseline_check = true;
        } else if arg == OsStr::new("--feedback-loop") {
            feedback_iterations = parse_usize_arg("--feedback-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-repair-loop") {
            feedback_repair_iterations = parse_usize_arg("--feedback-repair-loop", args.next())?;
        } else if arg == OsStr::new("--feedback-limit") {
            feedback_limit = parse_usize_arg("--feedback-limit", args.next())?;
        } else if arg == OsStr::new("--feedback-timeout") {
            feedback_timeout = parse_feedback_timeout(args.next())?;
        } else if arg == OsStr::new("--cargo-check-arg") {
            let value = args
                .next()
                .ok_or("--cargo-check-arg requires a following cargo check argument")?;
            cargo_check_args.push(
                value
                    .to_str()
                    .ok_or("--cargo-check-arg value must be valid UTF-8")?
                    .to_string(),
            );
        } else if arg == OsStr::new("--deny-warnings") {
            deny_warnings = true;
        } else if arg == OsStr::new("--feedback-report") {
            feedback_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--feedback-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--feedback-target-dir") {
            feedback_target_dir = Some(PathBuf::from(
                args.next()
                    .ok_or("--feedback-target-dir requires a following path")?,
            ));
        } else if arg == OsStr::new("--repair-report") {
            repair_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--repair-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--baseline-report") {
            baseline_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--baseline-report requires a following path")?,
            ));
            run_baseline_check = true;
        } else if arg == OsStr::new("--baseline-target-dir") {
            baseline_target_dir = Some(PathBuf::from(
                args.next()
                    .ok_or("--baseline-target-dir requires a following path")?,
            ));
            run_baseline_check = true;
        } else if arg == OsStr::new("--slice-report") {
            slice_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--slice-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--decision-log") {
            decision_log = Some(PathBuf::from(
                args.next()
                    .ok_or("--decision-log requires a following path")?,
            ));
        } else if arg == OsStr::new("--event-log") {
            event_log = Some(PathBuf::from(
                args.next().ok_or("--event-log requires a following path")?,
            ));
        } else if arg == OsStr::new("--validation-report") {
            validation_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--validation-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--preflight-report") {
            preflight_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--preflight-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--root") || arg == OsStr::new("--root-selector") {
            let value = args
                .next()
                .ok_or("--root requires a following function, method, or item selector")?;
            root_selectors.push(
                value
                    .to_str()
                    .ok_or("--root value must be valid UTF-8")?
                    .to_string(),
            );
        } else if arg == OsStr::new("--roots-file") {
            let path = PathBuf::from(
                args.next()
                    .ok_or("--roots-file requires a following path")?,
            );
            root_selectors.extend(read_root_selectors_file(&path)?);
        } else if arg == OsStr::new("--random-roots") {
            random_roots = Some(parse_usize_arg("--random-roots", args.next())?);
            batch_roots = true;
        } else if arg == OsStr::new("--random-root-package") {
            let value = args
                .next()
                .ok_or("--random-root-package requires a following package name")?;
            random_root_packages.push(
                value
                    .to_str()
                    .ok_or("--random-root-package value must be valid UTF-8")?
                    .to_string(),
            );
        } else if arg == OsStr::new("--random-seed") {
            random_seed = parse_u64_arg("--random-seed", args.next())?;
        } else if arg == OsStr::new("--batch-roots") {
            batch_roots = true;
        } else if arg == OsStr::new("--batch-report") {
            batch_report = Some(PathBuf::from(
                args.next()
                    .ok_or("--batch-report requires a following path")?,
            ));
        } else if arg == OsStr::new("--workspace-root") {
            workspace_root_arg = Some(PathBuf::from(
                args.next()
                    .ok_or("--workspace-root requires a following path")?,
            ));
        } else if arg == OsStr::new("--output") || arg == OsStr::new("--output-root") {
            output_root_arg = Some(PathBuf::from(
                args.next().ok_or("--output requires a following path")?,
            ));
        } else if arg == OsStr::new("--help") || arg == OsStr::new("-h") {
            println!("{}", usage());
            std::process::exit(0);
        } else {
            positional.push(PathBuf::from(&arg));
        }
    }

    let mut positional = positional.into_iter();
    let Some(workspace_root) = workspace_root_arg.or_else(|| positional.next()) else {
        return Err(usage().into());
    };
    let Some(output_root) = output_root_arg.or_else(|| positional.next()) else {
        return Err(usage().into());
    };
    if positional.next().is_some() {
        return Err(usage().into());
    }
    let workspace_root = normalize_workspace_root_arg(&workspace_root);

    if production_preset && !analyzer_mode_explicit {
        analyzer_mode = AnalyzerMode::production_default_for_build();
    }

    if production_should_add_locked_arg(production_preset, &workspace_root, &cargo_check_args) {
        cargo_check_args.push("--locked".to_string());
    }

    Ok(CliOptions {
        analyzer_mode,
        run_check,
        feedback_iterations,
        feedback_repair_iterations,
        feedback_limit,
        feedback_report,
        feedback_target_dir,
        feedback_timeout,
        cargo_check_args,
        deny_warnings,
        repair_report,
        run_baseline_check,
        allow_baseline_failures,
        baseline_report,
        baseline_target_dir,
        slice_report,
        decision_log,
        event_log,
        validation_report,
        run_preflight,
        preflight_report,
        production_preset,
        root_selectors,
        random_roots,
        random_root_packages,
        random_seed,
        batch_roots,
        batch_report,
        workspace_root,
        output_root,
    })
}

fn normalize_workspace_root_arg(path: &Path) -> PathBuf {
    if path.file_name() != Some(OsStr::new("Cargo.toml")) {
        return path.to_path_buf();
    }
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn apply_default_marked_package_scope(
    options: &mut CliOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    if !validation_runs_cargo_check(options)
        || cargo_args_have_package_scope(&options.cargo_check_args)
        || !options.root_selectors.is_empty()
        || options.random_roots.is_some()
        || options.batch_roots
    {
        return Ok(());
    }

    let packages = marked_workspace_packages(&options.workspace_root)?;
    if packages.is_empty() {
        return Ok(());
    }

    println!(
        "validation package scope: {}",
        packages
            .iter()
            .map(|package| format!("-p {package}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let mut scoped_args = package_scope_cargo_args(&packages);
    scoped_args.extend(options.cargo_check_args.clone());
    options.cargo_check_args = scoped_args;
    Ok(())
}

fn validation_runs_cargo_check(options: &CliOptions) -> bool {
    options.run_baseline_check
        || options.run_check
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0
}

fn package_scope_cargo_args(packages: &[String]) -> Vec<String> {
    packages
        .iter()
        .flat_map(|package| ["-p".to_string(), package.clone()])
        .collect()
}

fn cargo_args_have_package_scope(cargo_args: &[String]) -> bool {
    cargo_args.iter().any(|arg| {
        arg == "--workspace"
            || arg == "--all"
            || arg == "-p"
            || arg == "--package"
            || arg.starts_with("--package=")
            || (arg.starts_with("-p") && arg.len() > 2)
    })
}

fn production_should_add_locked_arg(
    production_preset: bool,
    workspace_root: &Path,
    cargo_check_args: &[String],
) -> bool {
    production_preset
        && workspace_root.join("Cargo.lock").exists()
        && !cargo_check_args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--locked" | "--frozen"))
}

fn run_batch_roots(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&options.output_root)?;
    let report_path = batch_report_path(options);
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, "")?;
    write_event_log(
        options,
        "batch",
        "initialized",
        "run multiple top-down roots from one source checkout",
        "batch mode amortizes resolver/analyzer loading across roots and keeps the source checkout unmodified",
        event_fields(&[
            ("batch_report", serde_json::json!(&report_path)),
            ("output_root", serde_json::json!(&options.output_root)),
            ("random_roots", serde_json::json!(options.random_roots)),
            ("root_selectors", serde_json::json!(&options.root_selectors)),
        ]),
    )?;

    let baseline = if options.run_baseline_check {
        write_event_log(
            options,
            "batch_baseline",
            "started",
            "run source baseline once for the batch",
            "one baseline report is reused when classifying generated-root failures",
            event_fields(&[(
                "target_dir",
                serde_json::json!(baseline_target_dir(options)),
            )]),
        )?;
        let report = run_baseline_check(options)?;
        write_baseline_report(options, &report)?;
        print_baseline(
            &report,
            options.feedback_limit,
            Some(&baseline_report_path(options)),
        );
        write_event_log(
            options,
            "batch_baseline",
            if report.success { "passed" } else { "failed" },
            "record source baseline once for the batch",
            if report.success {
                "source workspace baseline passed"
            } else {
                "source workspace baseline failed"
            },
            event_fields(&[
                ("errors", serde_json::json!(report.error_count())),
                ("warnings", serde_json::json!(report.warning_count())),
                ("duration_ms", serde_json::json!(report.duration_ms)),
            ]),
        )?;
        if !report.success && !options.allow_baseline_failures {
            return Err("source workspace failed baseline cargo check".into());
        }
        Some(report)
    } else {
        None
    };

    let direct_roots = if options.random_roots.is_none() && !options.root_selectors.is_empty() {
        direct_free_function_root_selectors(&options.root_selectors)
    } else {
        None
    };
    let (mut roots, resolver_session) = if let Some(roots) = direct_roots {
        write_event_log(
            options,
            "batch_resolver",
            "completed",
            "directly decode explicit free-function roots",
            "fully qualified function selectors do not need a pre-analyzer workspace scan",
            event_fields(&[(
                "roots",
                serde_json::json!(roots.iter().map(ToString::to_string).collect::<Vec<_>>()),
            )]),
        )?;
        (roots, None)
    } else {
        write_event_log(
            options,
            "batch_resolver",
            "started",
            "load narrow syntactic resolver for root selection",
            "selectors that need disambiguation parse the smallest available package scope before the wider top-down generation closure is loaded",
            event_fields(&[
                ("root_selectors", serde_json::json!(&options.root_selectors)),
                ("random_roots", serde_json::json!(options.random_roots)),
            ]),
        )?;
        let _heartbeat = start_event_heartbeat(
            options,
            "batch_resolver",
            "load narrow syntactic resolver for root selection",
            "selectors that need disambiguation parse the smallest available package scope before the wider top-down generation closure is loaded",
            event_fields(&[
                ("root_selectors", serde_json::json!(&options.root_selectors)),
                ("random_roots", serde_json::json!(options.random_roots)),
            ]),
        );
        let resolver_session_result = if options.random_roots.is_some() {
            GenerateSession::load_without_marker_targets(&options.workspace_root, AnalyzerMode::Syn)
        } else {
            GenerateSession::load_root_selector_resolver_without_marker_targets(
                &options.workspace_root,
                AnalyzerMode::Syn,
                &options.root_selectors,
            )
        };
        let resolver_session = match resolver_session_result {
            Ok(session) => session,
            Err(error) => {
                let reason = error.to_string();
                write_event_log(
                    options,
                    "batch_resolver",
                    "failed",
                    "load narrow syntactic resolver for root selection",
                    &reason,
                    event_fields(&[
                        ("root_selectors", serde_json::json!(&options.root_selectors)),
                        ("random_roots", serde_json::json!(options.random_roots)),
                    ]),
                )?;
                return Err(error);
            }
        };
        write_event_log(
            options,
            "batch_resolver",
            "completed",
            "load narrow syntactic resolver for root selection",
            "root selector resolver is ready",
            event_fields(&[(
                "selectable_roots",
                serde_json::json!(resolver_session.selectable_roots().len()),
            )]),
        )?;
        let mut roots = resolver_session.resolve_root_selectors(&options.root_selectors)?;
        if let Some(count) = options.random_roots {
            roots.extend(select_random_roots(
                random_root_candidates(
                    resolver_session.selectable_roots(),
                    &options.random_root_packages,
                ),
                count,
                options.random_seed,
            ));
        }
        (roots, Some(resolver_session))
    };
    roots = dedup_roots_preserve_order(roots);
    if roots.is_empty() {
        return Err(
            "--batch-roots requires at least one --root, --roots-file entry, or --random-roots"
                .into(),
        );
    }
    write_event_log(
        options,
        "batch_roots",
        "selected",
        "select batch root set",
        "each selected root is generated independently while sharing the loaded analyzer session",
        event_fields(&[
            ("root_count", serde_json::json!(roots.len())),
            (
                "roots",
                serde_json::json!(roots.iter().map(ToString::to_string).collect::<Vec<_>>()),
            ),
        ]),
    )?;

    let session_load_progress = |progress: GenerateSessionLoadProgress| {
        let _ = write_event_log(
            options,
            progress.event,
            progress.status,
            progress.decision,
            &progress.reason,
            load_progress_event_fields(&progress),
        );
    };

    let session = if options.analyzer_mode == AnalyzerMode::Syn {
        if let Some(resolver_session) = resolver_session {
            resolver_session
        } else {
            write_event_log(
                options,
                "batch_analyzer",
                "started",
                "load syntactic analyzer for direct roots",
                "direct root decoding skipped resolver loading, so generation loads the normal top-down project once",
                event_fields(&[("root_count", serde_json::json!(roots.len()))]),
            )?;
            let _heartbeat = start_event_heartbeat(
                options,
                "batch_analyzer",
                "load syntactic analyzer for direct roots",
                "direct root decoding skipped resolver loading, so generation loads the selected roots' dependency closure once",
                event_fields(&[
                    ("analyzer", serde_json::json!(options.analyzer_mode.as_str())),
                    ("root_count", serde_json::json!(roots.len())),
                    (
                        "roots",
                        serde_json::json!(roots.iter().map(ToString::to_string).collect::<Vec<_>>()),
                    ),
                ]),
            );
            match GenerateSession::load_with_selected_roots_with_progress(
                &options.workspace_root,
                options.analyzer_mode,
                &roots,
                Some(&session_load_progress),
            ) {
                Ok(session) => session,
                Err(error) => {
                    let reason = error.to_string();
                    write_event_log(
                        options,
                        "batch_analyzer",
                        "failed",
                        "load syntactic analyzer for direct roots",
                        &reason,
                        event_fields(&[
                            (
                                "analyzer",
                                serde_json::json!(options.analyzer_mode.as_str()),
                            ),
                            ("root_count", serde_json::json!(roots.len())),
                            (
                                "roots",
                                serde_json::json!(roots
                                    .iter()
                                    .map(ToString::to_string)
                                    .collect::<Vec<_>>()),
                            ),
                        ]),
                    )?;
                    return Err(error);
                }
            }
        }
    } else {
        write_event_log(
            options,
            "batch_analyzer",
            "started",
            "load requested analyzer once for selected roots",
            "one analyzer session is shared across all batch roots to avoid repeated cold starts",
            event_fields(&[
                (
                    "analyzer",
                    serde_json::json!(options.analyzer_mode.as_str()),
                ),
                ("root_count", serde_json::json!(roots.len())),
            ]),
        )?;
        let _heartbeat = start_event_heartbeat(
            options,
            "batch_analyzer",
            "load requested analyzer once for selected roots",
            "one analyzer session is shared across all batch roots to avoid repeated cold starts",
            event_fields(&[
                (
                    "analyzer",
                    serde_json::json!(options.analyzer_mode.as_str()),
                ),
                ("root_count", serde_json::json!(roots.len())),
                (
                    "roots",
                    serde_json::json!(roots.iter().map(ToString::to_string).collect::<Vec<_>>()),
                ),
            ]),
        );
        match GenerateSession::load_with_selected_roots_with_progress(
            &options.workspace_root,
            options.analyzer_mode,
            &roots,
            Some(&session_load_progress),
        ) {
            Ok(session) => session,
            Err(error) => {
                let reason = error.to_string();
                write_event_log(
                    options,
                    "batch_analyzer",
                    "failed",
                    "load requested analyzer once for selected roots",
                    &reason,
                    event_fields(&[
                        (
                            "analyzer",
                            serde_json::json!(options.analyzer_mode.as_str()),
                        ),
                        ("root_count", serde_json::json!(roots.len())),
                        (
                            "roots",
                            serde_json::json!(roots
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()),
                        ),
                    ]),
                )?;
                return Err(error);
            }
        }
    };
    let mut analyzer_fields = session_load_fields(&session);
    analyzer_fields.extend(event_fields(&[
        (
            "analyzer",
            serde_json::json!(session.analyzer().mode.as_str()),
        ),
        ("engine", serde_json::json!(&session.analyzer().engine)),
        ("notes", serde_json::json!(&session.analyzer().notes)),
    ]));
    write_event_log(
        options,
        "batch_analyzer",
        "completed",
        "load analyzer session for batch",
        "the loaded session will generate roots by downstream dependency closure",
        analyzer_fields,
    )?;
    println!(
        "analyzer: {} ({})",
        session.analyzer().mode.as_str(),
        session.analyzer().engine
    );
    for note in &session.analyzer().notes {
        println!("  analyzer note: {note}");
    }

    println!(
        "batch roots: {} slice(s); report: {}",
        roots.len(),
        report_path.display()
    );
    for root in &roots {
        println!("  batch root: {root}");
    }
    for (index, root) in roots.iter().enumerate() {
        let output_root = options.output_root.join(batch_output_dir_name(index, root));
        println!(
            "batch {}/{} start: {} -> {}",
            index + 1,
            roots.len(),
            root,
            output_root.display()
        );
        write_event_log(
            options,
            "batch_root",
            "started",
            "generate and validate one selected root",
            "the root is treated as the top-level open-source surface and only downstream dependencies are retained",
            event_fields(&[
                ("index", serde_json::json!(index + 1)),
                ("total", serde_json::json!(roots.len())),
                ("root", serde_json::json!(root.to_string())),
                ("output_root", serde_json::json!(&output_root)),
            ]),
        )?;
        let started = std::time::Instant::now();
        let row = match run_batch_root(options, &session, root, &output_root, baseline.as_ref()) {
            Ok(mut row) => {
                row.duration_ms = elapsed_ms(started);
                row
            }
            Err(error) => BatchRootReport {
                root: root.to_string(),
                output_root,
                status: "failed".to_string(),
                files_written: None,
                production_status: None,
                production_hazards: None,
                rendered_usage_used: None,
                rendered_usage_blocked_by_unknown: None,
                rendered_usage_invalid: None,
                usage_semantic_proof_status: None,
                usage_semantic_proof_required_callables: None,
                usage_semantic_proof_required_items: None,
                usage_semantic_proven_callables: None,
                usage_semantic_proven_items: None,
                usage_semantic_unproven_callables: None,
                usage_semantic_unproven_items: None,
                usage_semantic_cfg_inactive_callables: None,
                usage_semantic_cfg_inactive_items: None,
                usage_semantic_source_file_pruned_callables: None,
                usage_semantic_source_file_pruned_items: None,
                usage_semantic_structural_pruned_items: None,
                usage_semantic_unmapped_callables: None,
                usage_semantic_unmapped_items: None,
                usage_semantic_failed_reference_query_callables: None,
                usage_semantic_failed_reference_query_items: None,
                semantic_proof_status: None,
                semantic_file_budget: None,
                semantic_method_call_budget: None,
                semantic_path_budget: None,
                semantic_source_files: None,
                semantic_analyzed_files: None,
                semantic_failed_files: None,
                semantic_skipped_files: None,
                semantic_unresolved_method_calls: None,
                semantic_unqueried_method_calls: None,
                semantic_unresolved_paths: None,
                semantic_unqueried_paths: None,
                selected_root_semantic_source_files: None,
                selected_root_semantic_analyzed_files: None,
                selected_root_semantic_failed_files: None,
                selected_root_semantic_skipped_files: None,
                selected_root_unresolved_method_calls: None,
                selected_root_unqueried_method_calls: None,
                selected_root_unresolved_paths: None,
                selected_root_unqueried_paths: None,
                preflight_errors: None,
                preflight_warnings: None,
                check_success: None,
                check_errors: None,
                check_warnings: None,
                feedback_diagnostics: None,
                feedback_widening_candidates: None,
                feedback_widening_hazards: None,
                feedback_resolution_matched_roots: None,
                feedback_resolution_skipped_no_match: None,
                feedback_baseline_compared: None,
                feedback_baseline_known_errors: None,
                feedback_baseline_new_errors: None,
                source_api_mismatch_candidates: None,
                glob_import_context_entries: None,
                glob_import_missing_export_candidates: None,
                glob_import_private_upstream_candidates: None,
                duration_ms: elapsed_ms(started),
                artifact_error: None,
                error: Some(error.to_string()),
            },
        };
        println!(
            "batch {}/{}: {} {}",
            index + 1,
            roots.len(),
            row.status,
            row.root
        );
        write_event_log(
            options,
            "batch_root",
            &row.status,
            "record final row for one selected root",
            row.error
                .as_deref()
                .unwrap_or("batch root finished with recorded validation status"),
            event_fields(&[
                ("index", serde_json::json!(index + 1)),
                ("total", serde_json::json!(roots.len())),
                ("root", serde_json::json!(&row.root)),
                ("output_root", serde_json::json!(&row.output_root)),
                ("duration_ms", serde_json::json!(row.duration_ms)),
                ("files_written", serde_json::json!(row.files_written)),
                (
                    "rendered_usage_used",
                    serde_json::json!(row.rendered_usage_used),
                ),
                (
                    "rendered_usage_blocked_by_unknown",
                    serde_json::json!(row.rendered_usage_blocked_by_unknown),
                ),
                (
                    "rendered_usage_invalid",
                    serde_json::json!(row.rendered_usage_invalid),
                ),
                (
                    "usage_semantic_proof_status",
                    serde_json::json!(row.usage_semantic_proof_status),
                ),
                (
                    "usage_semantic_unproven_callables",
                    serde_json::json!(row.usage_semantic_unproven_callables),
                ),
                (
                    "usage_semantic_unproven_items",
                    serde_json::json!(row.usage_semantic_unproven_items),
                ),
                (
                    "usage_semantic_cfg_inactive_callables",
                    serde_json::json!(row.usage_semantic_cfg_inactive_callables),
                ),
                (
                    "usage_semantic_cfg_inactive_items",
                    serde_json::json!(row.usage_semantic_cfg_inactive_items),
                ),
                (
                    "usage_semantic_source_file_pruned_callables",
                    serde_json::json!(row.usage_semantic_source_file_pruned_callables),
                ),
                (
                    "usage_semantic_source_file_pruned_items",
                    serde_json::json!(row.usage_semantic_source_file_pruned_items),
                ),
                (
                    "usage_semantic_structural_pruned_items",
                    serde_json::json!(row.usage_semantic_structural_pruned_items),
                ),
                ("check_success", serde_json::json!(row.check_success)),
                ("check_errors", serde_json::json!(row.check_errors)),
                ("check_warnings", serde_json::json!(row.check_warnings)),
                (
                    "feedback_diagnostics",
                    serde_json::json!(row.feedback_diagnostics),
                ),
                (
                    "feedback_widening_candidates",
                    serde_json::json!(row.feedback_widening_candidates),
                ),
                (
                    "feedback_widening_hazards",
                    serde_json::json!(row.feedback_widening_hazards),
                ),
                (
                    "feedback_resolution_matched_roots",
                    serde_json::json!(row.feedback_resolution_matched_roots),
                ),
                (
                    "feedback_resolution_skipped_no_match",
                    serde_json::json!(row.feedback_resolution_skipped_no_match),
                ),
                (
                    "feedback_baseline_compared",
                    serde_json::json!(row.feedback_baseline_compared),
                ),
                (
                    "feedback_baseline_known_errors",
                    serde_json::json!(row.feedback_baseline_known_errors),
                ),
                (
                    "feedback_baseline_new_errors",
                    serde_json::json!(row.feedback_baseline_new_errors),
                ),
                (
                    "source_api_mismatch_candidates",
                    serde_json::json!(row.source_api_mismatch_candidates),
                ),
                (
                    "glob_import_context_entries",
                    serde_json::json!(row.glob_import_context_entries),
                ),
                (
                    "glob_import_missing_export_candidates",
                    serde_json::json!(row.glob_import_missing_export_candidates),
                ),
                (
                    "glob_import_private_upstream_candidates",
                    serde_json::json!(row.glob_import_private_upstream_candidates),
                ),
            ]),
        )?;
        append_batch_report_row(&report_path, &row)?;
    }
    write_event_log(
        options,
        "batch",
        "completed",
        "finish batch root run",
        "all selected roots were generated and their rows appended to the batch report",
        event_fields(&[
            ("root_count", serde_json::json!(roots.len())),
            ("batch_report", serde_json::json!(&report_path)),
        ]),
    )?;
    Ok(())
}

fn dedup_roots_preserve_order(roots: Vec<RootId>) -> Vec<RootId> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for root in roots {
        if seen.insert(root.to_string()) {
            deduped.push(root);
        }
    }
    deduped
}

fn random_root_candidates(roots: Vec<RootId>, packages: &[String]) -> Vec<RootId> {
    if packages.is_empty() {
        return roots;
    }
    let packages = packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    roots
        .into_iter()
        .filter(|root| packages.contains(root.package()))
        .collect()
}

fn run_batch_root(
    options: &CliOptions,
    session: &GenerateSession,
    root: &RootId,
    output_root: &Path,
    baseline: Option<&CheckReport>,
) -> Result<BatchRootReport, Box<dyn std::error::Error>> {
    let mut diagnostics = Vec::<CheckDiagnostic>::new();
    let attempts = batch_root_attempts(options);
    let mut last_report = None;
    let mut last_preflight = None;
    let root_options = batch_root_live_artifact_options(options, root, output_root);
    initialize_event_log(&root_options)?;
    write_batch_root_event_log(
        options,
        &root_options,
        root,
        output_root,
        "batch_root_context",
        "started",
        "start shared-analyzer batch root artifact logging",
        "this root-local event log records generation, validation, repair, and final decisions for one selected top-down root",
        event_fields(&[
            ("attempts", serde_json::json!(attempts)),
            (
                "shared_analyzer",
                serde_json::json!(options.analyzer_mode != AnalyzerMode::Syn),
            ),
        ]),
    )?;

    macro_rules! root_event {
        ($event:expr, $status:expr, $decision:expr, $reason:expr, $fields:expr $(,)?) => {
            write_batch_root_event_log(
                options,
                &root_options,
                root,
                output_root,
                $event,
                $status,
                $decision,
                $reason,
                $fields,
            )
        };
    }
    macro_rules! root_heartbeat {
        ($event:expr, $decision:expr, $reason:expr, $fields:expr $(,)?) => {
            start_batch_root_event_heartbeat(
                options,
                &root_options,
                root,
                output_root,
                $event,
                $decision,
                $reason,
                $fields,
            )
        };
    }

    for attempt in 1..=attempts {
        root_event!(
            "batch_generation",
            "started",
            "generate one top-down root slice",
            "generation starts from the selected root and walks only downstream dependencies",
            event_fields(&[
                ("root", serde_json::json!(root.to_string())),
                ("attempt", serde_json::json!(attempt)),
                ("attempts", serde_json::json!(attempts)),
                ("output_root", serde_json::json!(output_root)),
                (
                    "diagnostics_available",
                    serde_json::json!(diagnostics.len()),
                ),
            ]),
        )?;
        let _heartbeat = root_heartbeat!(
            "batch_generation",
            "generate one top-down root slice",
            "generation starts from the selected root and walks only downstream dependencies",
            event_fields(&[
                ("root", serde_json::json!(root.to_string())),
                ("attempt", serde_json::json!(attempt)),
                ("attempts", serde_json::json!(attempts)),
                ("output_root", serde_json::json!(output_root)),
                (
                    "diagnostics_available",
                    serde_json::json!(diagnostics.len()),
                ),
            ]),
        );
        let report =
            match session.generate(output_root.to_path_buf(), &[root.clone()], &diagnostics) {
                Ok(report) => report,
                Err(error) => {
                    let reason = error.to_string();
                    root_event!(
                        "batch_generation",
                        "failed",
                        "generate one top-down root slice",
                        &reason,
                        event_fields(&[
                            ("root", serde_json::json!(root.to_string())),
                            ("attempt", serde_json::json!(attempt)),
                            ("attempts", serde_json::json!(attempts)),
                            ("output_root", serde_json::json!(output_root)),
                        ]),
                    )?;
                    return Err(error);
                }
            };
        root_event!(
            "batch_generation",
            "completed",
            "render selected root plus downstream closure",
            "generation produced a candidate slice workspace for this root",
            event_fields(&[
                ("root", serde_json::json!(root.to_string())),
                ("attempt", serde_json::json!(attempt)),
                ("packages", serde_json::json!(&report.packages)),
                ("files_written", serde_json::json!(report.files_written)),
                (
                    "production_status",
                    serde_json::json!(&report.production.status),
                ),
                (
                    "production_hazards",
                    serde_json::json!(report.production.hazards.len()),
                ),
                (
                    "rendered_usage",
                    serde_json::json!(rendered_usage_contract(&report).summary_fields()),
                ),
                (
                    "reachable_callables",
                    serde_json::json!(report.reachable.len()),
                ),
                (
                    "reachable_items",
                    serde_json::json!(report.reachable_items.len()),
                ),
                ("total_ms", serde_json::json!(report.timings.total_ms)),
                ("reduce_ms", serde_json::json!(report.timings.reduce_ms)),
                ("render_ms", serde_json::json!(report.timings.render_ms)),
            ]),
        )?;
        write_generate_report(&report, &output_root.join("slice-report.json"))?;
        root_event!(
            "batch_artifact",
            "written",
            "write generated slice report for root",
            "slice-report.json captures the top-down closure, usage classification, production hazards, and rendered file inventory for this root",
            event_fields(&[
                (
                    "report_path",
                    serde_json::json!(output_root.join("slice-report.json")),
                ),
                ("files_written", serde_json::json!(report.files_written)),
            ]),
        )?;
        let rendered_usage_contract = rendered_usage_contract(&report);
        let semantic_proof_block = options
            .production_preset
            .then(|| semantic_proof_block_reason(report.analyzer.semantic.as_ref()))
            .flatten();
        last_report = Some(report);
        if !rendered_usage_contract.invalid.is_empty() {
            root_event!(
                "batch_gate",
                "failed",
                "stop batch root on rendered usage contract",
                "rendered source contains invalid usage decisions",
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                    (
                        "invalid_preview",
                        serde_json::json!(rendered_usage_contract.invalid_preview()),
                    ),
                ]),
            )?;
            return finish_batch_root(
                options,
                root,
                output_root,
                "rendered_usage_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                baseline,
                Some(format!(
                    "rendered source contains invalid usage decisions: {}",
                    rendered_usage_contract.invalid_preview()
                )),
            );
        }
        if let Some(reason) = semantic_proof_block {
            root_event!(
                "batch_gate",
                "failed",
                "stop batch root on semantic proof gate",
                &reason,
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                ]),
            )?;
            return finish_batch_root(
                options,
                root,
                output_root,
                "semantic_proof_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                baseline,
                Some(reason),
            );
        }
        if let Err(error) = refresh_generated_lockfile_for_output(options, output_root) {
            let reason = error.to_string();
            root_event!(
                "batch_gate",
                "failed",
                "stop batch root on lockfile refresh",
                &reason,
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                ]),
            )?;
            return finish_batch_root(
                options,
                root,
                output_root,
                "lockfile_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                baseline,
                Some(reason),
            );
        }

        if options.run_preflight || attempts > 1 {
            root_event!(
                "batch_preflight",
                "started",
                "run fast structural validation for batch root",
                "preflight catches malformed generated workspaces before cargo check",
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                    (
                        "manifest",
                        serde_json::json!(output_root.join("Cargo.toml")),
                    ),
                ]),
            )?;
            let _heartbeat = root_heartbeat!(
                "batch_preflight",
                "run fast structural validation for batch root",
                "preflight catches malformed generated workspaces before cargo check",
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                    (
                        "manifest",
                        serde_json::json!(output_root.join("Cargo.toml")),
                    ),
                ]),
            );
            let preflight = match preflight_workspace(PreflightOptions {
                manifest_path: output_root.join("Cargo.toml"),
            }) {
                Ok(preflight) => preflight,
                Err(error) => {
                    let reason = error.to_string();
                    let _ = root_event!(
                        "batch_preflight",
                        "failed",
                        "run fast structural validation for batch root",
                        &reason,
                        event_fields(&[
                            ("root", serde_json::json!(root.to_string())),
                            ("attempt", serde_json::json!(attempt)),
                            (
                                "manifest",
                                serde_json::json!(output_root.join("Cargo.toml")),
                            ),
                        ]),
                    );
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "preflight_error",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        None,
                        baseline,
                        Some(reason),
                    );
                }
            };
            write_preflight_report(&preflight, &output_root.join("slice-preflight.json"))?;
            root_event!(
                "batch_preflight",
                if preflight.success {
                    "passed"
                } else {
                    "failed"
                },
                "run fast structural validation for batch root",
                if preflight.success {
                    "generated workspace passed preflight"
                } else {
                    "generated workspace failed preflight"
                },
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                    ("errors", serde_json::json!(preflight.error_count())),
                    ("warnings", serde_json::json!(preflight.warning_count())),
                    ("packages", serde_json::json!(preflight.packages)),
                    ("rust_files", serde_json::json!(preflight.rust_files)),
                ]),
            )?;
            if !preflight.success {
                root_event!(
                    "batch_gate",
                    "failed",
                    "stop batch root on preflight gate",
                    "generated workspace failed preflight",
                    event_fields(&[
                        ("root", serde_json::json!(root.to_string())),
                        ("attempt", serde_json::json!(attempt)),
                        ("errors", serde_json::json!(preflight.error_count())),
                        ("warnings", serde_json::json!(preflight.warning_count())),
                    ]),
                )?;
                return finish_batch_root(
                    options,
                    root,
                    output_root,
                    "preflight_failed",
                    last_report.as_ref(),
                    Some(&preflight),
                    None,
                    baseline,
                    None,
                );
            }
            last_preflight = Some(preflight);
        }

        if !batch_runs_check(options) {
            root_event!(
                "batch_gate",
                "passed",
                "finish batch root after generation",
                "cargo check was not requested for this batch root",
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("attempt", serde_json::json!(attempt)),
                ]),
            )?;
            return finish_batch_root(
                options,
                root,
                output_root,
                "generated",
                last_report.as_ref(),
                last_preflight.as_ref(),
                None,
                baseline,
                None,
            );
        }

        root_event!(
            "batch_check",
            "started",
            "run cargo check for generated batch root",
            "cargo check validates that the top-down generated workspace builds",
            event_fields(&[
                ("root", serde_json::json!(root.to_string())),
                ("attempt", serde_json::json!(attempt)),
                (
                    "target_dir",
                    serde_json::json!(batch_feedback_target_dir(options)),
                ),
                (
                    "cargo_args",
                    serde_json::json!(batch_cargo_args(options, root)),
                ),
            ]),
        )?;
        let _heartbeat = root_heartbeat!(
            "batch_check",
            "run cargo check for generated batch root",
            "cargo check validates that the top-down generated workspace builds",
            event_fields(&[
                ("root", serde_json::json!(root.to_string())),
                ("attempt", serde_json::json!(attempt)),
                (
                    "target_dir",
                    serde_json::json!(batch_feedback_target_dir(options)),
                ),
                (
                    "cargo_args",
                    serde_json::json!(batch_cargo_args(options, root)),
                ),
            ]),
        );
        let check = match check_workspace(CheckOptions {
            manifest_path: output_root.join("Cargo.toml"),
            target_dir: Some(batch_feedback_target_dir(options)),
            timeout: options.feedback_timeout,
            cargo_args: batch_cargo_args(options, root),
        }) {
            Ok(check) => check,
            Err(error) => {
                let reason = error.to_string();
                let _ = root_event!(
                    "batch_check",
                    "failed",
                    "run cargo check for generated batch root",
                    &reason,
                    event_fields(&[
                        ("root", serde_json::json!(root.to_string())),
                        ("attempt", serde_json::json!(attempt)),
                        (
                            "target_dir",
                            serde_json::json!(batch_feedback_target_dir(options)),
                        ),
                        (
                            "cargo_args",
                            serde_json::json!(batch_cargo_args(options, root)),
                        ),
                    ]),
                );
                return finish_batch_root(
                    options,
                    root,
                    output_root,
                    "check_error",
                    last_report.as_ref(),
                    last_preflight.as_ref(),
                    None,
                    baseline,
                    Some(reason),
                );
            }
        };
        write_report(&check, &output_root.join("slice-feedback.json"))?;
        let accepted = if options.feedback_repair_iterations > 0 {
            feedback_repair_is_accepted(&check, baseline, options.deny_warnings)
        } else {
            feedback_is_accepted(&check, baseline, options.deny_warnings)
        };
        root_event!(
            "batch_check",
            if accepted { "accepted" } else { "failed" },
            "classify cargo check result for generated batch root",
            if accepted {
                "generated workspace passed the requested check policy"
            } else {
                "generated workspace did not pass the requested check policy"
            },
            event_fields(&[
                ("root", serde_json::json!(root.to_string())),
                ("attempt", serde_json::json!(attempt)),
                ("success", serde_json::json!(check.success)),
                ("timed_out", serde_json::json!(check.timed_out)),
                ("exit_code", serde_json::json!(check.exit_code)),
                ("duration_ms", serde_json::json!(check.duration_ms)),
                ("errors", serde_json::json!(check.error_count())),
                ("warnings", serde_json::json!(check.warning_count())),
                (
                    "report_path",
                    serde_json::json!(output_root.join("slice-feedback.json")),
                ),
            ]),
        )?;
        if accepted {
            return finish_batch_root(
                options,
                root,
                output_root,
                "accepted",
                last_report.as_ref(),
                last_preflight.as_ref(),
                Some(&check),
                baseline,
                None,
            );
        }
        if options.feedback_repair_iterations > 0 {
            let mut repaired_check = check.clone();
            let mut saw_deferred_warning_allows = false;
            for repair_attempt in 1..=options.feedback_repair_iterations {
                root_event!(
                    "batch_repair",
                    "started",
                    "apply conservative repair to generated root",
                    "repair uses compiler diagnostics only to adjust generated output before rechecking the same top-down root",
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("repair_attempt", serde_json::json!(repair_attempt)),
                        (
                            "diagnostics",
                            serde_json::json!(repaired_check.diagnostics.len()),
                        ),
                        ("errors", serde_json::json!(repaired_check.error_count())),
                        ("warnings", serde_json::json!(repaired_check.warning_count())),
                    ]),
                )?;
                let repair = match repair_workspace(RepairOptions {
                    output_root: output_root.to_path_buf(),
                    diagnostics: repaired_check.diagnostics.clone(),
                }) {
                    Ok(repair) => repair,
                    Err(error) => {
                        let reason = error.to_string();
                        let _ = root_event!(
                            "batch_repair",
                            "failed",
                            "apply conservative repair to generated root",
                            &reason,
                            event_fields(&[
                                ("attempt", serde_json::json!(attempt)),
                                ("repair_attempt", serde_json::json!(repair_attempt)),
                            ]),
                        );
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "repair_error",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            Some(reason),
                        );
                    }
                };
                if let Err(error) =
                    write_repair_report(&repair, &output_root.join("slice-repair.json"))
                {
                    let reason = error.to_string();
                    let _ = root_event!(
                        "batch_repair",
                        "failed",
                        "write generated repair report",
                        &reason,
                        event_fields(&[
                            ("attempt", serde_json::json!(attempt)),
                            ("repair_attempt", serde_json::json!(repair_attempt)),
                        ]),
                    );
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "repair_report_error",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        Some(&repaired_check),
                        baseline,
                        Some(reason),
                    );
                }
                saw_deferred_warning_allows |= repair_has_deferred_warning_allows(&repair);
                root_event!(
                    "batch_repair",
                    if repair.total_changes() > 0 {
                        "changed"
                    } else {
                        "unchanged"
                    },
                    "record conservative repair result for generated root",
                    if repair.total_changes() > 0 {
                        "repair changed generated files and the root must pass preflight and cargo check again"
                    } else {
                        "no safe repair matched the current diagnostics"
                    },
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("repair_attempt", serde_json::json!(repair_attempt)),
                        ("removed_items", serde_json::json!(repair.removed_items)),
                        ("removed_imports", serde_json::json!(repair.removed_imports)),
                        (
                            "normalized_paths",
                            serde_json::json!(repair.normalized_paths)
                        ),
                        (
                            "applied_suggestions",
                            serde_json::json!(repair.applied_suggestions),
                        ),
                        (
                            "added_dead_code_allows",
                            serde_json::json!(repair.added_dead_code_allows),
                        ),
                        (
                            "deferred_dead_code_allows",
                            serde_json::json!(repair.deferred_dead_code_allows),
                        ),
                        ("changed_files", serde_json::json!(repair.changed_files)),
                        ("total_changes", serde_json::json!(repair.total_changes())),
                        (
                            "repair_report",
                            serde_json::json!(output_root.join("slice-repair.json")),
                        ),
                    ]),
                )?;
                if repair.total_changes() == 0 {
                    break;
                }
                root_event!(
                    "batch_repair_preflight",
                    "started",
                    "run preflight after conservative repair",
                    "a repair must preserve generated workspace structure before cargo check is rerun",
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("repair_attempt", serde_json::json!(repair_attempt)),
                        (
                            "manifest",
                            serde_json::json!(output_root.join("Cargo.toml")),
                        ),
                    ]),
                )?;
                let preflight = match preflight_workspace(PreflightOptions {
                    manifest_path: output_root.join("Cargo.toml"),
                }) {
                    Ok(preflight) => preflight,
                    Err(error) => {
                        let reason = error.to_string();
                        let _ = root_event!(
                            "batch_repair_preflight",
                            "failed",
                            "run preflight after conservative repair",
                            &reason,
                            event_fields(&[
                                ("attempt", serde_json::json!(attempt)),
                                ("repair_attempt", serde_json::json!(repair_attempt)),
                            ]),
                        );
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "preflight_error",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            Some(reason),
                        );
                    }
                };
                if let Err(error) =
                    write_preflight_report(&preflight, &output_root.join("slice-preflight.json"))
                {
                    let reason = error.to_string();
                    let _ = root_event!(
                        "batch_repair_preflight",
                        "failed",
                        "write preflight report after conservative repair",
                        &reason,
                        event_fields(&[
                            ("attempt", serde_json::json!(attempt)),
                            ("repair_attempt", serde_json::json!(repair_attempt)),
                        ]),
                    );
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "preflight_report_error",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        Some(&repaired_check),
                        baseline,
                        Some(reason),
                    );
                }
                root_event!(
                    "batch_repair_preflight",
                    if preflight.success {
                        "passed"
                    } else {
                        "failed"
                    },
                    "run preflight after conservative repair",
                    if preflight.success {
                        "repaired workspace passed structural validation"
                    } else {
                        "repaired workspace failed structural validation"
                    },
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("repair_attempt", serde_json::json!(repair_attempt)),
                        ("errors", serde_json::json!(preflight.error_count())),
                        ("warnings", serde_json::json!(preflight.warning_count())),
                        (
                            "preflight_report",
                            serde_json::json!(output_root.join("slice-preflight.json")),
                        ),
                    ]),
                )?;
                if !preflight.success {
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "preflight_failed",
                        last_report.as_ref(),
                        Some(&preflight),
                        Some(&check),
                        baseline,
                        Some("batch repair produced a structurally invalid workspace".to_string()),
                    );
                }
                last_preflight = Some(preflight);
                root_event!(
                    "batch_repair_check",
                    "started",
                    "rerun cargo check after conservative repair",
                    "repair acceptance requires a fresh compiler check of the repaired generated workspace",
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("repair_attempt", serde_json::json!(repair_attempt)),
                        (
                            "target_dir",
                            serde_json::json!(batch_feedback_target_dir(options)),
                        ),
                        (
                            "cargo_args",
                            serde_json::json!(batch_cargo_args(options, root)),
                        ),
                    ]),
                )?;
                repaired_check = match check_workspace(CheckOptions {
                    manifest_path: output_root.join("Cargo.toml"),
                    target_dir: Some(batch_feedback_target_dir(options)),
                    timeout: options.feedback_timeout,
                    cargo_args: batch_cargo_args(options, root),
                }) {
                    Ok(check) => check,
                    Err(error) => {
                        let reason = error.to_string();
                        let _ = root_event!(
                            "batch_repair_check",
                            "failed",
                            "rerun cargo check after conservative repair",
                            &reason,
                            event_fields(&[
                                ("attempt", serde_json::json!(attempt)),
                                ("repair_attempt", serde_json::json!(repair_attempt)),
                            ]),
                        );
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "check_error",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            Some(reason),
                        );
                    }
                };
                if let Err(error) =
                    write_report(&repaired_check, &output_root.join("slice-feedback.json"))
                {
                    let reason = error.to_string();
                    let _ = root_event!(
                        "batch_repair_check",
                        "failed",
                        "write compiler feedback report after conservative repair",
                        &reason,
                        event_fields(&[
                            ("attempt", serde_json::json!(attempt)),
                            ("repair_attempt", serde_json::json!(repair_attempt)),
                        ]),
                    );
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "check_report_error",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        Some(&repaired_check),
                        baseline,
                        Some(reason),
                    );
                }
                let repaired_accepted =
                    feedback_repair_is_accepted(&repaired_check, baseline, options.deny_warnings);
                root_event!(
                    "batch_repair_check",
                    if repaired_accepted {
                        "accepted"
                    } else {
                        "failed"
                    },
                    "classify cargo check after conservative repair",
                    if repaired_accepted {
                        "repaired generated workspace passed the requested check policy"
                    } else {
                        "repaired generated workspace still did not pass the requested check policy"
                    },
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("repair_attempt", serde_json::json!(repair_attempt)),
                        ("success", serde_json::json!(repaired_check.success)),
                        ("timed_out", serde_json::json!(repaired_check.timed_out)),
                        ("exit_code", serde_json::json!(repaired_check.exit_code)),
                        ("duration_ms", serde_json::json!(repaired_check.duration_ms)),
                        ("errors", serde_json::json!(repaired_check.error_count())),
                        (
                            "warnings",
                            serde_json::json!(repaired_check.warning_count())
                        ),
                        (
                            "report_path",
                            serde_json::json!(output_root.join("slice-feedback.json")),
                        ),
                    ]),
                )?;
                if repaired_accepted {
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "accepted",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        Some(&repaired_check),
                        baseline,
                        None,
                    );
                }
            }
            if should_run_deferred_warning_repair(
                &repaired_check,
                baseline,
                options.deny_warnings,
                saw_deferred_warning_allows,
            ) {
                root_event!(
                    "batch_deferred_warning_repair",
                    "started",
                    "apply deferred warning repair after feedback loop",
                    "deferred warning repair runs only when dead-code allow cleanup may make the repaired generated workspace acceptable",
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("errors", serde_json::json!(repaired_check.error_count())),
                        ("warnings", serde_json::json!(repaired_check.warning_count())),
                    ]),
                )?;
                let repair = match repair_workspace(RepairOptions {
                    output_root: output_root.to_path_buf(),
                    diagnostics: repaired_check.diagnostics.clone(),
                }) {
                    Ok(repair) => repair,
                    Err(error) => {
                        let reason = error.to_string();
                        let _ = root_event!(
                            "batch_deferred_warning_repair",
                            "failed",
                            "apply deferred warning repair after feedback loop",
                            &reason,
                            event_fields(&[("attempt", serde_json::json!(attempt))]),
                        );
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "repair_error",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            Some(reason),
                        );
                    }
                };
                if let Err(error) =
                    write_repair_report(&repair, &output_root.join("slice-repair.json"))
                {
                    let reason = error.to_string();
                    let _ = root_event!(
                        "batch_deferred_warning_repair",
                        "failed",
                        "write deferred warning repair report",
                        &reason,
                        event_fields(&[("attempt", serde_json::json!(attempt))]),
                    );
                    return finish_batch_root(
                        options,
                        root,
                        output_root,
                        "repair_report_error",
                        last_report.as_ref(),
                        last_preflight.as_ref(),
                        Some(&repaired_check),
                        baseline,
                        Some(reason),
                    );
                }
                root_event!(
                    "batch_deferred_warning_repair",
                    if repair.total_changes() > 0 {
                        "changed"
                    } else {
                        "unchanged"
                    },
                    "record deferred warning repair result",
                    if repair.total_changes() > 0 {
                        "deferred warning repair changed generated files and requires preflight plus cargo check"
                    } else {
                        "deferred warning repair had no safe generated change to apply"
                    },
                    event_fields(&[
                        ("attempt", serde_json::json!(attempt)),
                        ("removed_items", serde_json::json!(repair.removed_items)),
                        ("removed_imports", serde_json::json!(repair.removed_imports)),
                        (
                            "normalized_paths",
                            serde_json::json!(repair.normalized_paths)
                        ),
                        (
                            "applied_suggestions",
                            serde_json::json!(repair.applied_suggestions),
                        ),
                        (
                            "deferred_dead_code_allows",
                            serde_json::json!(repair.deferred_dead_code_allows),
                        ),
                        ("changed_files", serde_json::json!(repair.changed_files)),
                        ("total_changes", serde_json::json!(repair.total_changes())),
                        (
                            "repair_report",
                            serde_json::json!(output_root.join("slice-repair.json")),
                        ),
                    ]),
                )?;
                if repair.total_changes() > 0 {
                    root_event!(
                        "batch_deferred_warning_preflight",
                        "started",
                        "run preflight after deferred warning repair",
                        "deferred warning repair must preserve generated workspace structure before cargo check",
                        event_fields(&[("attempt", serde_json::json!(attempt))]),
                    )?;
                    let preflight = match preflight_workspace(PreflightOptions {
                        manifest_path: output_root.join("Cargo.toml"),
                    }) {
                        Ok(preflight) => preflight,
                        Err(error) => {
                            let reason = error.to_string();
                            let _ = root_event!(
                                "batch_deferred_warning_preflight",
                                "failed",
                                "run preflight after deferred warning repair",
                                &reason,
                                event_fields(&[("attempt", serde_json::json!(attempt))]),
                            );
                            return finish_batch_root(
                                options,
                                root,
                                output_root,
                                "preflight_error",
                                last_report.as_ref(),
                                last_preflight.as_ref(),
                                Some(&repaired_check),
                                baseline,
                                Some(reason),
                            );
                        }
                    };
                    if let Err(error) = write_preflight_report(
                        &preflight,
                        &output_root.join("slice-preflight.json"),
                    ) {
                        let reason = error.to_string();
                        let _ = root_event!(
                            "batch_deferred_warning_preflight",
                            "failed",
                            "write deferred warning preflight report",
                            &reason,
                            event_fields(&[("attempt", serde_json::json!(attempt))]),
                        );
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "preflight_report_error",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            Some(reason),
                        );
                    }
                    root_event!(
                        "batch_deferred_warning_preflight",
                        if preflight.success {
                            "passed"
                        } else {
                            "failed"
                        },
                        "run preflight after deferred warning repair",
                        if preflight.success {
                            "deferred-warning repaired workspace passed structural validation"
                        } else {
                            "deferred-warning repaired workspace failed structural validation"
                        },
                        event_fields(&[
                            ("attempt", serde_json::json!(attempt)),
                            ("errors", serde_json::json!(preflight.error_count())),
                            ("warnings", serde_json::json!(preflight.warning_count())),
                        ]),
                    )?;
                    if !preflight.success {
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "preflight_failed",
                            last_report.as_ref(),
                            Some(&preflight),
                            Some(&repaired_check),
                            baseline,
                            Some(
                                "batch warning repair produced a structurally invalid workspace"
                                    .to_string(),
                            ),
                        );
                    }
                    last_preflight = Some(preflight);
                    root_event!(
                        "batch_deferred_warning_check",
                        "started",
                        "rerun cargo check after deferred warning repair",
                        "deferred warning repair acceptance requires fresh compiler feedback",
                        event_fields(&[
                            ("attempt", serde_json::json!(attempt)),
                            (
                                "target_dir",
                                serde_json::json!(batch_feedback_target_dir(options)),
                            ),
                            (
                                "cargo_args",
                                serde_json::json!(batch_cargo_args(options, root)),
                            ),
                        ]),
                    )?;
                    repaired_check = match check_workspace(CheckOptions {
                        manifest_path: output_root.join("Cargo.toml"),
                        target_dir: Some(batch_feedback_target_dir(options)),
                        timeout: options.feedback_timeout,
                        cargo_args: batch_cargo_args(options, root),
                    }) {
                        Ok(check) => check,
                        Err(error) => {
                            let reason = error.to_string();
                            let _ = root_event!(
                                "batch_deferred_warning_check",
                                "failed",
                                "rerun cargo check after deferred warning repair",
                                &reason,
                                event_fields(&[("attempt", serde_json::json!(attempt))]),
                            );
                            return finish_batch_root(
                                options,
                                root,
                                output_root,
                                "check_error",
                                last_report.as_ref(),
                                last_preflight.as_ref(),
                                Some(&repaired_check),
                                baseline,
                                Some(reason),
                            );
                        }
                    };
                    if let Err(error) =
                        write_report(&repaired_check, &output_root.join("slice-feedback.json"))
                    {
                        let reason = error.to_string();
                        let _ = root_event!(
                            "batch_deferred_warning_check",
                            "failed",
                            "write deferred warning feedback report",
                            &reason,
                            event_fields(&[("attempt", serde_json::json!(attempt))]),
                        );
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "check_report_error",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            Some(reason),
                        );
                    }
                    let deferred_accepted = feedback_repair_is_accepted(
                        &repaired_check,
                        baseline,
                        options.deny_warnings,
                    );
                    root_event!(
                        "batch_deferred_warning_check",
                        if deferred_accepted {
                            "accepted"
                        } else {
                            "failed"
                        },
                        "classify cargo check after deferred warning repair",
                        if deferred_accepted {
                            "deferred-warning repaired workspace passed the requested check policy"
                        } else {
                            "deferred-warning repaired workspace still did not pass the requested check policy"
                        },
                        event_fields(&[
                            ("attempt", serde_json::json!(attempt)),
                            ("success", serde_json::json!(repaired_check.success)),
                            ("timed_out", serde_json::json!(repaired_check.timed_out)),
                            ("exit_code", serde_json::json!(repaired_check.exit_code)),
                            ("duration_ms", serde_json::json!(repaired_check.duration_ms)),
                            ("errors", serde_json::json!(repaired_check.error_count())),
                            (
                                "warnings",
                                serde_json::json!(repaired_check.warning_count())
                            ),
                        ]),
                    )?;
                    if deferred_accepted {
                        return finish_batch_root(
                            options,
                            root,
                            output_root,
                            "accepted",
                            last_report.as_ref(),
                            last_preflight.as_ref(),
                            Some(&repaired_check),
                            baseline,
                            None,
                        );
                    }
                }
            }
            if attempt == attempts || repaired_check.error_count() == 0 {
                return finish_batch_root(
                    options,
                    root,
                    output_root,
                    "check_failed",
                    last_report.as_ref(),
                    last_preflight.as_ref(),
                    Some(&repaired_check),
                    baseline,
                    Some("repaired workspace did not pass batch feedback gate".to_string()),
                );
            }
            diagnostics.extend(repaired_check.diagnostics);
            continue;
        }
        if attempt == attempts || check.error_count() == 0 {
            return finish_batch_root(
                options,
                root,
                output_root,
                "check_failed",
                last_report.as_ref(),
                last_preflight.as_ref(),
                Some(&check),
                baseline,
                Some("generated workspace did not pass batch feedback gate".to_string()),
            );
        }
        diagnostics.extend(check.diagnostics);
    }

    finish_batch_root(
        options,
        root,
        output_root,
        "failed",
        last_report.as_ref(),
        last_preflight.as_ref(),
        None,
        baseline,
        Some("batch loop ended without a final report".to_string()),
    )
}

fn batch_row_from_reports(
    options: &CliOptions,
    root: &RootId,
    output_root: &Path,
    status: &str,
    report: Option<&GenerateReport>,
    preflight: Option<&PreflightReport>,
    check: Option<&CheckReport>,
    baseline: Option<&CheckReport>,
    error: Option<String>,
) -> BatchRootReport {
    let rendered_usage = report.map(rendered_usage_contract);
    let semantic = report.and_then(|report| report.analyzer.semantic.as_ref());
    let usage_semantic_proof = report.map(|report| &report.usage.semantic_proof);
    let usage_semantic_summary = usage_semantic_proof.map(|proof| &proof.summary);
    let feedback_summary =
        check.map(|check| feedback_diagnostic_summary_for_check(options, check, 0));
    let baseline_comparison =
        check.map(|check| feedback_baseline_error_comparison(check, baseline));
    BatchRootReport {
        root: root.to_string(),
        output_root: output_root.to_path_buf(),
        status: status.to_string(),
        files_written: report.map(|report| report.files_written),
        production_status: report.map(|report| report.production.status.clone()),
        production_hazards: report.map(|report| report.production.hazards.len()),
        rendered_usage_used: rendered_usage.as_ref().map(|contract| contract.used),
        rendered_usage_blocked_by_unknown: rendered_usage
            .as_ref()
            .map(|contract| contract.blocked_by_unknown),
        rendered_usage_invalid: rendered_usage
            .as_ref()
            .map(|contract| contract.invalid.len()),
        usage_semantic_proof_status: usage_semantic_proof.map(|proof| proof.status.clone()),
        usage_semantic_proof_required_callables: usage_semantic_summary
            .map(|summary| summary.proof_required_callables),
        usage_semantic_proof_required_items: usage_semantic_summary
            .map(|summary| summary.proof_required_items),
        usage_semantic_proven_callables: usage_semantic_summary
            .map(|summary| summary.proven_callables),
        usage_semantic_proven_items: usage_semantic_summary.map(|summary| summary.proven_items),
        usage_semantic_unproven_callables: usage_semantic_summary
            .map(|summary| summary.unproven_callables),
        usage_semantic_unproven_items: usage_semantic_summary.map(|summary| summary.unproven_items),
        usage_semantic_cfg_inactive_callables: usage_semantic_summary
            .map(|summary| summary.cfg_inactive_callables),
        usage_semantic_cfg_inactive_items: usage_semantic_summary
            .map(|summary| summary.cfg_inactive_items),
        usage_semantic_source_file_pruned_callables: usage_semantic_summary
            .map(|summary| summary.source_file_pruned_callables),
        usage_semantic_source_file_pruned_items: usage_semantic_summary
            .map(|summary| summary.source_file_pruned_items),
        usage_semantic_structural_pruned_items: usage_semantic_summary
            .map(|summary| summary.structural_pruned_items),
        usage_semantic_unmapped_callables: usage_semantic_summary
            .map(|summary| summary.unmapped_callables),
        usage_semantic_unmapped_items: usage_semantic_summary.map(|summary| summary.unmapped_items),
        usage_semantic_failed_reference_query_callables: usage_semantic_summary
            .map(|summary| summary.failed_reference_query_callables),
        usage_semantic_failed_reference_query_items: usage_semantic_summary
            .map(|summary| summary.failed_reference_query_items),
        semantic_proof_status: semantic.map(semantic_proof_status),
        semantic_file_budget: semantic.map(|semantic| semantic.file_budget),
        semantic_method_call_budget: semantic.map(|semantic| semantic.method_call_budget),
        semantic_path_budget: semantic.map(|semantic| semantic.path_budget),
        semantic_source_files: semantic.map(|semantic| semantic.source_files),
        semantic_analyzed_files: semantic.map(|semantic| semantic.analyzed_files),
        semantic_failed_files: semantic.map(|semantic| semantic.failed_files),
        semantic_skipped_files: semantic.map(|semantic| semantic.skipped_files),
        semantic_unresolved_method_calls: semantic.map(|semantic| semantic.unresolved_method_calls),
        semantic_unqueried_method_calls: semantic.map(|semantic| semantic.unqueried_method_calls),
        semantic_unresolved_paths: semantic.map(|semantic| semantic.unresolved_paths),
        semantic_unqueried_paths: semantic.map(|semantic| semantic.unqueried_paths),
        selected_root_semantic_source_files: semantic
            .map(|semantic| semantic.selected_root_source_files),
        selected_root_semantic_analyzed_files: semantic
            .map(|semantic| semantic.selected_root_analyzed_files),
        selected_root_semantic_failed_files: semantic
            .map(|semantic| semantic.selected_root_failed_files),
        selected_root_semantic_skipped_files: semantic
            .map(|semantic| semantic.selected_root_skipped_files),
        selected_root_unresolved_method_calls: semantic
            .map(|semantic| semantic.selected_root_unresolved_method_calls),
        selected_root_unqueried_method_calls: semantic
            .map(|semantic| semantic.selected_root_unqueried_method_calls),
        selected_root_unresolved_paths: semantic
            .map(|semantic| semantic.selected_root_unresolved_paths),
        selected_root_unqueried_paths: semantic
            .map(|semantic| semantic.selected_root_unqueried_paths),
        preflight_errors: preflight.map(PreflightReport::error_count),
        preflight_warnings: preflight.map(PreflightReport::warning_count),
        check_success: check.map(|check| check.success),
        check_errors: check.map(CheckReport::error_count),
        check_warnings: check.map(CheckReport::warning_count),
        feedback_diagnostics: feedback_summary.as_ref().map(|summary| summary.diagnostics),
        feedback_widening_candidates: feedback_summary
            .as_ref()
            .map(|summary| summary.widening_candidates),
        feedback_widening_hazards: feedback_summary
            .as_ref()
            .map(|summary| summary.widening_hazards),
        feedback_resolution_matched_roots: feedback_summary
            .as_ref()
            .map(|summary| summary.resolution_matched_roots),
        feedback_resolution_skipped_no_match: feedback_summary
            .as_ref()
            .map(|summary| summary.resolution_skipped_no_match),
        feedback_baseline_compared: baseline_comparison.map(|comparison| comparison.compared),
        feedback_baseline_known_errors: baseline_comparison
            .map(|comparison| comparison.known_errors),
        feedback_baseline_new_errors: baseline_comparison.map(|comparison| comparison.new_errors),
        source_api_mismatch_candidates: feedback_summary
            .as_ref()
            .map(|summary| summary.source_api_mismatch_candidates),
        glob_import_context_entries: feedback_summary
            .as_ref()
            .map(|summary| summary.glob_import_context_entries),
        glob_import_missing_export_candidates: feedback_summary
            .as_ref()
            .map(|summary| summary.glob_import_missing_export_candidates),
        glob_import_private_upstream_candidates: feedback_summary
            .as_ref()
            .map(|summary| summary.glob_import_private_upstream_candidates),
        duration_ms: 0,
        artifact_error: None,
        error,
    }
}

fn finish_batch_root(
    options: &CliOptions,
    root: &RootId,
    output_root: &Path,
    status: &str,
    report: Option<&GenerateReport>,
    preflight: Option<&PreflightReport>,
    check: Option<&CheckReport>,
    baseline: Option<&CheckReport>,
    error: Option<String>,
) -> Result<BatchRootReport, Box<dyn std::error::Error>> {
    let root_options = batch_root_artifact_options(options, root, output_root);
    let mut row = batch_row_from_reports(
        &root_options,
        root,
        output_root,
        status,
        report,
        preflight,
        check,
        baseline,
        error.clone(),
    );
    if let Some(report) = report {
        if let Err(artifact_error) = write_batch_root_artifacts(
            options,
            &root_options,
            root,
            output_root,
            status,
            report,
            preflight,
            check,
            baseline,
            &error,
        ) {
            let artifact_error = artifact_error.to_string();
            row.artifact_error = Some(artifact_error.clone());
            let _ = write_event_log(
                options,
                "batch_root_artifacts",
                "failed",
                "write per-root batch artifact logs",
                &artifact_error,
                event_fields(&[
                    ("root", serde_json::json!(root.to_string())),
                    ("output_root", serde_json::json!(output_root)),
                    ("status", serde_json::json!(status)),
                ]),
            );
        }
    }
    Ok(row)
}

fn write_batch_root_artifacts(
    batch_options: &CliOptions,
    root_options: &CliOptions,
    root: &RootId,
    output_root: &Path,
    status: &str,
    report: &GenerateReport,
    preflight: Option<&PreflightReport>,
    check: Option<&CheckReport>,
    baseline: Option<&CheckReport>,
    error: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    preserve_batch_root_live_event_log(output_root, root_options)?;
    initialize_event_log_if_absent(root_options)?;
    write_event_log(
        root_options,
        "batch_root_context",
        status,
        "record shared-analyzer batch root artifact context",
        "this per-root log was produced from a batch run that loaded rust-analyzer once and generated this root by top-down dependency closure",
        event_fields(&[
            ("root", serde_json::json!(root.to_string())),
            (
                "batch_output_root",
                serde_json::json!(&batch_options.output_root),
            ),
            ("root_output_root", serde_json::json!(output_root)),
            (
                "shared_analyzer",
                serde_json::json!(batch_options.analyzer_mode != AnalyzerMode::Syn),
            ),
            ("status", serde_json::json!(status)),
            ("error", serde_json::json!(error)),
        ]),
    )?;
    let mut validation = batch_root_validation_report(
        root_options,
        status,
        report,
        preflight,
        check,
        baseline,
        error,
    );
    write_feedback_diagnostics_event(root_options, &validation)?;
    finish_validation(
        root_options,
        &mut validation,
        batch_root_validation_status(status),
        error.as_deref(),
    )?;
    write_decision_log(
        root_options,
        report,
        Some(&validation),
        batch_root_validation_status(status),
        error.as_deref(),
    )
}

fn batch_root_live_artifact_options(
    options: &CliOptions,
    root: &RootId,
    output_root: &Path,
) -> CliOptions {
    let mut root_options = batch_root_artifact_options(options, root, output_root);
    root_options.event_log = Some(batch_root_live_event_log_path(output_root));
    root_options
}

fn batch_root_live_event_log_path(output_root: &Path) -> PathBuf {
    let parent = output_root.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output_root
        .file_name()
        .and_then(OsStr::to_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("batch-root");
    parent.join(format!("{file_name}-slice-events.jsonl"))
}

fn preserve_batch_root_live_event_log(
    output_root: &Path,
    root_options: &CliOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(final_path) = event_log_path(root_options) else {
        return Ok(());
    };
    let live_path = batch_root_live_event_log_path(output_root);
    if live_path == final_path || !live_path.exists() || final_path.exists() {
        return Ok(());
    }
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(live_path, final_path)?;
    Ok(())
}

fn batch_root_artifact_options(
    options: &CliOptions,
    root: &RootId,
    output_root: &Path,
) -> CliOptions {
    let mut root_options = options.clone();
    root_options.output_root = output_root.to_path_buf();
    root_options.root_selectors = vec![root.to_string()];
    root_options.random_roots = None;
    root_options.random_root_packages = Vec::new();
    root_options.batch_roots = false;
    root_options.batch_report = None;
    root_options.slice_report = Some(output_root.join("slice-report.json"));
    root_options.decision_log = Some(output_root.join("slice-decision-log.json"));
    root_options.validation_report = Some(output_root.join("slice-validation.json"));
    root_options.event_log = Some(output_root.join("slice-events.jsonl"));
    root_options.preflight_report = Some(output_root.join("slice-preflight.json"));
    root_options.feedback_report = Some(output_root.join("slice-feedback.json"));
    root_options.repair_report = Some(output_root.join("slice-repair.json"));
    root_options
}

fn batch_root_validation_status(status: &str) -> &'static str {
    match status {
        "accepted" | "generated" => "accepted",
        _ => "rejected",
    }
}

fn batch_root_validation_report(
    options: &CliOptions,
    status: &str,
    report: &GenerateReport,
    preflight: Option<&PreflightReport>,
    check: Option<&CheckReport>,
    baseline: Option<&CheckReport>,
    error: &Option<String>,
) -> ValidationReport {
    let mut validation = ValidationReport::new(options);
    validation.gates.push(ValidationGateReport {
        name: "generation".to_string(),
        status: "passed".to_string(),
        reason: "batch root slice workspace was generated from the shared analyzer session"
            .to_string(),
        report_path: slice_report_path(options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });

    let rendered_usage = rendered_usage_contract(report);
    validation.gates.push(ValidationGateReport {
        name: "rendered_usage_contract".to_string(),
        status: if rendered_usage.invalid.is_empty() {
            "passed".to_string()
        } else {
            "failed".to_string()
        },
        reason: if rendered_usage.invalid.is_empty() {
            format!(
                "rendered source contains {} used and {} blocked_by_unknown symbols",
                rendered_usage.used, rendered_usage.blocked_by_unknown
            )
        } else {
            format!(
                "rendered source contains invalid usage decisions: {}",
                rendered_usage.invalid_preview()
            )
        },
        report_path: slice_report_path(options),
        error_count: Some(rendered_usage.invalid.len()),
        warning_count: Some(rendered_usage.blocked_by_unknown),
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });

    record_semantic_proof_gate(options, &mut validation, report);
    record_production_readiness_gate(options, &mut validation, &report.production, "batch");

    if let Some(preflight) = preflight {
        validation.gates.push(ValidationGateReport {
            name: "preflight".to_string(),
            status: if preflight.success {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            reason: if preflight.success {
                "generated workspace passed fast structural validation".to_string()
            } else {
                "generated workspace failed fast structural validation".to_string()
            },
            report_path: maybe_preflight_report_path(options),
            error_count: Some(preflight.error_count()),
            warning_count: Some(preflight.warning_count()),
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
    }

    if let Some(check) = check {
        let baseline_comparison = feedback_baseline_error_comparison(check, baseline);
        validation.feedback_baseline_compared = Some(baseline_comparison.compared);
        validation.feedback_baseline_known_errors = Some(baseline_comparison.known_errors);
        validation.feedback_baseline_new_errors = Some(baseline_comparison.new_errors);
        validation.add_check_gate(
            "feedback",
            if check.success && status == "accepted" {
                "accepted"
            } else {
                "failed"
            },
            if check.success && status == "accepted" {
                "generated workspace cargo check passed all feedback gates"
            } else {
                "generated workspace did not pass the batch feedback gate"
            },
            check,
            Some(options.output_root.join("slice-feedback.json")),
            semantic_hazard_warning_count(&check.diagnostics, None),
        );
    }

    if let Some(error) = error {
        validation.gates.push(ValidationGateReport {
            name: "batch_result".to_string(),
            status: "failed".to_string(),
            reason: error.clone(),
            report_path: slice_report_path(options),
            error_count: Some(1),
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
    }

    record_final_production_readiness(options, &mut validation);

    validation
}

fn semantic_proof_status(semantic: &SemanticReport) -> String {
    let workspace_limited = semantic.skipped_files > 0
        || semantic.unqueried_method_calls > 0
        || semantic.unqueried_paths > 0;

    if semantic.selected_root_source_files > 0 {
        if semantic.selected_root_failed_files > 0 {
            return "selected_root_failed".to_string();
        }
        if semantic.selected_root_skipped_files > 0
            || semantic.selected_root_unqueried_method_calls > 0
            || semantic.selected_root_unqueried_paths > 0
        {
            return "selected_root_limited".to_string();
        }
        if workspace_limited {
            return "selected_root_complete_workspace_limited".to_string();
        }
        return "complete".to_string();
    }

    if semantic.failed_files > 0 {
        return "workspace_failed".to_string();
    }
    if workspace_limited {
        return "workspace_limited".to_string();
    }
    if semantic.source_files == 0 {
        return "empty".to_string();
    }
    "complete".to_string()
}

fn record_semantic_proof_gate(
    options: &CliOptions,
    validation: &mut ValidationReport,
    report: &GenerateReport,
) -> Option<String> {
    let semantic = report.analyzer.semantic.as_ref();
    let status = semantic
        .map(semantic_proof_status)
        .unwrap_or_else(|| "not_available".to_string());
    let reason = semantic_proof_reason(semantic);
    let blocks = options.production_preset && semantic_proof_status_blocks(&status);
    validation.gates.push(ValidationGateReport {
        name: "semantic_proof".to_string(),
        status: if blocks {
            "failed".to_string()
        } else {
            status.clone()
        },
        reason: reason.clone(),
        report_path: slice_report_path(options),
        error_count: semantic.map(semantic_proof_error_count),
        warning_count: semantic.map(semantic_proof_warning_count),
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    blocks.then_some(reason)
}

fn semantic_proof_block_reason(semantic: Option<&SemanticReport>) -> Option<String> {
    let status = semantic
        .map(semantic_proof_status)
        .unwrap_or_else(|| "not_available".to_string());
    semantic_proof_status_blocks(&status).then(|| semantic_proof_reason(semantic))
}

fn semantic_proof_status_blocks(status: &str) -> bool {
    matches!(
        status,
        "not_available"
            | "empty"
            | "selected_root_failed"
            | "selected_root_limited"
            | "workspace_failed"
            | "workspace_limited"
    )
}

fn semantic_proof_reason(semantic: Option<&SemanticReport>) -> String {
    let Some(semantic) = semantic else {
        return "rust-analyzer semantic proof is not available".to_string();
    };
    match semantic_proof_status(semantic).as_str() {
        "complete" => "rust-analyzer semantic proof covered the selected slice".to_string(),
        "selected_root_complete_workspace_limited" => format!(
            "selected root semantic proof is complete; wider workspace budget remains limited ({} skipped file(s), {} unqueried method call(s), {} unqueried path(s))",
            semantic.skipped_files, semantic.unqueried_method_calls, semantic.unqueried_paths
        ),
        "selected_root_failed" => format!(
            "selected root semantic proof failed in {} root file(s)",
            semantic.selected_root_failed_files
        ),
        "selected_root_limited" => format!(
            "selected root semantic proof is budget-limited ({} skipped root file(s), {} unqueried root method call(s), {} unqueried root path(s))",
            semantic.selected_root_skipped_files,
            semantic.selected_root_unqueried_method_calls,
            semantic.selected_root_unqueried_paths
        ),
        "workspace_failed" => format!(
            "workspace semantic proof failed in {} file(s)",
            semantic.failed_files
        ),
        "workspace_limited" => format!(
            "workspace semantic proof is budget-limited ({} skipped file(s), {} unqueried method call(s), {} unqueried path(s))",
            semantic.skipped_files, semantic.unqueried_method_calls, semantic.unqueried_paths
        ),
        "empty" => "rust-analyzer semantic proof did not find Rust source files".to_string(),
        _ => "rust-analyzer semantic proof state is unknown".to_string(),
    }
}

fn semantic_proof_error_count(semantic: &SemanticReport) -> usize {
    if semantic.selected_root_source_files > 0 {
        semantic.selected_root_failed_files
            + semantic.selected_root_skipped_files
            + semantic.selected_root_unqueried_method_calls
            + semantic.selected_root_unqueried_paths
    } else {
        semantic.failed_files
            + semantic.skipped_files
            + semantic.unqueried_method_calls
            + semantic.unqueried_paths
    }
}

fn semantic_proof_warning_count(semantic: &SemanticReport) -> usize {
    if semantic.selected_root_source_files > 0 {
        semantic.skipped_files + semantic.unqueried_method_calls + semantic.unqueried_paths
    } else {
        semantic.unresolved_method_calls + semantic.unresolved_paths
    }
}

fn batch_runs_check(options: &CliOptions) -> bool {
    options.run_check || options.feedback_iterations > 0 || options.feedback_repair_iterations > 0
}

fn batch_root_attempts(options: &CliOptions) -> usize {
    let attempts = options
        .feedback_iterations
        .max(options.feedback_repair_iterations)
        .max(usize::from(options.run_check))
        .max(1);
    if batch_runs_check(options)
        && (options.feedback_iterations > 0 || options.feedback_repair_iterations > 0)
    {
        attempts.saturating_add(1)
    } else {
        attempts
    }
}

fn batch_feedback_target_dir(options: &CliOptions) -> PathBuf {
    options
        .feedback_target_dir
        .clone()
        .unwrap_or_else(|| options.output_root.join("target-feedback"))
}

fn batch_cargo_args(options: &CliOptions, root: &RootId) -> Vec<String> {
    if cargo_args_have_package_scope(&options.cargo_check_args) {
        return options.cargo_check_args.clone();
    }
    let mut args = vec!["-p".to_string(), root.package().to_string()];
    args.extend(options.cargo_check_args.clone());
    args
}

fn batch_report_path(options: &CliOptions) -> PathBuf {
    options
        .batch_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("batch-report.jsonl"))
}

fn append_batch_report_row(
    report_path: &Path,
    row: &BatchRootReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(report_path)?;
    serde_json::to_writer(&mut file, row)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn batch_output_dir_name(index: usize, root: &RootId) -> String {
    let mut slug = root
        .to_string()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    let slug = slug.trim_matches('_');
    let slug = if slug.is_empty() { "root" } else { slug };
    let max_len = slug.len().min(96);
    format!("{:04}-{}", index + 1, &slug[..max_len])
}

fn select_random_roots(mut roots: Vec<RootId>, count: usize, seed: u64) -> Vec<RootId> {
    roots.sort_by_key(|root| stable_root_score(root, seed));
    roots.truncate(count.min(roots.len()));
    roots
}

fn stable_root_score(root: &RootId, seed: u64) -> u64 {
    let mut hash = seed ^ 0xcbf2_9ce4_8422_2325;
    for byte in root.to_string().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}

fn elapsed_ms(started: std::time::Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn parse_usize_arg(
    flag: &str,
    value: Option<std::ffi::OsString>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let value = value.ok_or_else(|| format!("{flag} requires a following number"))?;
    let value = value
        .to_str()
        .ok_or_else(|| format!("{flag} value must be valid UTF-8"))?;
    let parsed = value.parse::<usize>()?;
    if parsed == 0 {
        return Err(format!("{flag} must be greater than zero").into());
    }
    Ok(parsed)
}

fn parse_u64_arg(
    flag: &str,
    value: Option<std::ffi::OsString>,
) -> Result<u64, Box<dyn std::error::Error>> {
    let value = value.ok_or_else(|| format!("{flag} requires a following number"))?;
    let value = value
        .to_str()
        .ok_or_else(|| format!("{flag} value must be valid UTF-8"))?;
    Ok(value.parse::<u64>()?)
}

fn read_root_selectors_file(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect())
}

fn parse_feedback_timeout(
    value: Option<std::ffi::OsString>,
) -> Result<Option<Duration>, Box<dyn std::error::Error>> {
    let value = value.ok_or("--feedback-timeout requires a following number of seconds")?;
    let value = value
        .to_str()
        .ok_or("--feedback-timeout value must be valid UTF-8")?;
    let seconds = value.parse::<u64>()?;
    Ok((seconds > 0).then(|| Duration::from_secs(seconds)))
}

#[derive(Debug, Clone)]
struct PlainCheckOutput {
    status: String,
    stdout_excerpt: String,
    stderr_excerpt: String,
}

#[derive(Debug)]
struct PlainCheckFailure {
    output: PlainCheckOutput,
}

impl std::fmt::Display for PlainCheckFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "generated workspace failed cargo check with {}",
            self.output.status
        )
    }
}

impl std::error::Error for PlainCheckFailure {}

fn run_plain_check(options: &CliOptions) -> Result<PlainCheckOutput, Box<dyn std::error::Error>> {
    let manifest_path = absolute_path(&options.output_root.join("Cargo.toml"))?;
    let working_dir = manifest_working_dir(&manifest_path);
    let output = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .args(&options.cargo_check_args)
        .current_dir(&working_dir)
        .output()?;
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    let check_output = PlainCheckOutput {
        status: output.status.to_string(),
        stdout_excerpt: command_output_excerpt(&output.stdout),
        stderr_excerpt: command_output_excerpt(&output.stderr),
    };
    if !output.status.success() {
        return Err(Box::new(PlainCheckFailure {
            output: check_output,
        }));
    }
    Ok(check_output)
}

fn command_output_excerpt(output: &[u8]) -> String {
    const MAX_CHARS: usize = 12_000;
    let text = String::from_utf8_lossy(output);
    if text.chars().count() <= MAX_CHARS {
        return text.into_owned();
    }
    let tail = text
        .chars()
        .rev()
        .take(MAX_CHARS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("[truncated to last {MAX_CHARS} chars]\n{tail}")
}

fn run_plain_check_gate(
    options: &CliOptions,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match run_plain_check(options) {
        Ok(output) => {
            write_event_log(
                options,
                "check",
                "passed",
                "run plain generated workspace cargo check",
                "generated workspace cargo check passed",
                event_fields(&[
                    ("status", serde_json::json!(output.status)),
                    ("stderr_excerpt", serde_json::json!(output.stderr_excerpt)),
                ]),
            )?;
            validation.gates.push(ValidationGateReport {
                name: "check".to_string(),
                status: "passed".to_string(),
                reason: "generated workspace cargo check passed".to_string(),
                report_path: None,
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            Ok(())
        }
        Err(error) => {
            let reason = error.to_string();
            let failure = error.downcast_ref::<PlainCheckFailure>();
            write_event_log(
                options,
                "check",
                "failed",
                "run plain generated workspace cargo check",
                &reason,
                event_fields(&[
                    (
                        "status",
                        serde_json::json!(failure.map(|failure| failure.output.status.clone())),
                    ),
                    (
                        "stdout_excerpt",
                        serde_json::json!(
                            failure.map(|failure| failure.output.stdout_excerpt.clone())
                        ),
                    ),
                    (
                        "stderr_excerpt",
                        serde_json::json!(
                            failure.map(|failure| failure.output.stderr_excerpt.clone())
                        ),
                    ),
                ]),
            )?;
            validation.gates.push(ValidationGateReport {
                name: "check".to_string(),
                status: "failed".to_string(),
                reason: reason.clone(),
                report_path: None,
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            finish_validation(options, validation, "rejected", Some(&reason))?;
            Err(reason.into())
        }
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn manifest_working_dir(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn run_preflight(options: &CliOptions) -> Result<PreflightReport, Box<dyn std::error::Error>> {
    let report_path = preflight_report_path(options);
    let report = preflight_workspace(PreflightOptions {
        manifest_path: options.output_root.join("Cargo.toml"),
    })?;
    write_preflight_report(&report, &report_path)?;
    print_preflight(&report, options.feedback_limit, &report_path);
    Ok(report)
}

fn run_baseline_check(options: &CliOptions) -> Result<CheckReport, Box<dyn std::error::Error>> {
    let target_dir = baseline_target_dir(options);
    println!(
        "baseline: cargo check --message-format=json (target {})",
        target_dir.display()
    );
    run_baseline_check_with_args(options, options.cargo_check_args.clone(), target_dir)
}

fn run_baseline_check_with_args(
    options: &CliOptions,
    cargo_args: Vec<String>,
    target_dir: PathBuf,
) -> Result<CheckReport, Box<dyn std::error::Error>> {
    check_workspace(CheckOptions {
        manifest_path: options.workspace_root.join("Cargo.toml"),
        target_dir: Some(target_dir),
        timeout: options.feedback_timeout,
        cargo_args,
    })
}

fn write_baseline_report(
    options: &CliOptions,
    report: &CheckReport,
) -> Result<(), Box<dyn std::error::Error>> {
    write_report(report, &baseline_report_path(options))
}

fn baseline_report_path(options: &CliOptions) -> PathBuf {
    options
        .baseline_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-baseline.json"))
}

fn slice_report_path(options: &CliOptions) -> Option<PathBuf> {
    options.slice_report.clone().or_else(|| {
        options
            .production_preset
            .then(|| options.output_root.join("slice-report.json"))
    })
}

fn decision_log_path(options: &CliOptions) -> Option<PathBuf> {
    options.decision_log.clone().or_else(|| {
        options
            .production_preset
            .then(|| options.output_root.join("slice-decision-log.json"))
    })
}

fn event_log_path(options: &CliOptions) -> Option<PathBuf> {
    options.event_log.clone().or_else(|| {
        if options.batch_roots {
            Some(default_batch_event_log_path(&options.output_root))
        } else {
            options
                .production_preset
                .then(|| default_event_log_path(&options.output_root))
        }
    })
}

fn default_batch_event_log_path(output_root: &Path) -> PathBuf {
    output_root.join("batch-events.jsonl")
}

fn default_event_log_path(output_root: &Path) -> PathBuf {
    let mut path = output_root.to_path_buf();
    let file_name = output_root
        .file_name()
        .and_then(OsStr::to_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("slice-output");
    path.set_file_name(format!("{file_name}-slice-events.jsonl"));
    path
}

fn preflight_report_path(options: &CliOptions) -> PathBuf {
    options
        .preflight_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-preflight.json"))
}

fn maybe_preflight_report_path(options: &CliOptions) -> Option<PathBuf> {
    (options.run_preflight
        || options.feedback_iterations > 0
        || options.feedback_repair_iterations > 0)
        .then(|| preflight_report_path(options))
}

fn feedback_report_path(options: &CliOptions) -> Option<PathBuf> {
    (options.feedback_iterations > 0 || options.feedback_repair_iterations > 0).then(|| {
        options
            .feedback_report
            .clone()
            .unwrap_or_else(|| options.output_root.join("slice-feedback.json"))
    })
}

fn decision_log_feedback_report_path(options: &CliOptions) -> Option<PathBuf> {
    feedback_report_path(options).or_else(|| {
        options.run_check.then(|| {
            let path = options
                .feedback_report
                .clone()
                .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));
            path.exists().then_some(path)
        })?
    })
}

fn repair_report_path(options: &CliOptions) -> Option<PathBuf> {
    (options.feedback_repair_iterations > 0).then(|| {
        options
            .repair_report
            .clone()
            .unwrap_or_else(|| options.output_root.join("slice-repair.json"))
    })
}

fn validation_report_path(options: &CliOptions) -> Option<PathBuf> {
    options.validation_report.clone().or_else(|| {
        (options.production_preset
            || options.run_baseline_check
            || options.run_preflight
            || options.feedback_iterations > 0
            || options.feedback_repair_iterations > 0
            || options.run_check)
            .then(|| options.output_root.join("slice-validation.json"))
    })
}

fn uncovered_validation_targets(
    targets: &[GeneratedTargetReport],
    cargo_args: &[String],
) -> Vec<String> {
    targets
        .iter()
        .filter_map(|target| {
            let coverage_kind = validation_target_kind(target);
            if let Some(target_kind) = coverage_kind {
                if !validation_target_is_covered(target_kind, &target.name, cargo_args) {
                    return Some(format!(
                        "{} {} {}",
                        target.package, target_kind, target.name
                    ));
                }
            }
            let missing_features = uncovered_target_required_features(target, cargo_args);
            let feature_kind = coverage_kind.or_else(|| required_feature_target_kind(target));
            (!missing_features.is_empty()).then(|| {
                let target_kind = feature_kind.unwrap_or("target");
                format!(
                    "{} {} {} requires feature(s): {}",
                    target.package,
                    target_kind,
                    target.name,
                    missing_features.join(", ")
                )
            })
        })
        .collect()
}

fn validation_target_kind(target: &GeneratedTargetReport) -> Option<&'static str> {
    ["example", "test", "bench"]
        .into_iter()
        .find(|kind| target.kind.iter().any(|target_kind| target_kind == *kind))
}

fn required_feature_target_kind(target: &GeneratedTargetReport) -> Option<&'static str> {
    ["bin", "example", "test", "bench"]
        .into_iter()
        .find(|kind| target.kind.iter().any(|target_kind| target_kind == *kind))
}

fn validation_target_is_covered(kind: &str, name: &str, cargo_args: &[String]) -> bool {
    if cargo_args.iter().any(|arg| arg == "--all-targets") {
        return true;
    }
    if cargo_args
        .iter()
        .any(|arg| arg == target_kind_plural_flag(kind))
    {
        return true;
    }
    let singular_flag = format!("--{kind}");
    for index in 0..cargo_args.len() {
        let arg = &cargo_args[index];
        if arg == &singular_flag && cargo_args.get(index + 1).is_some_and(|value| value == name) {
            return true;
        }
        if arg == &format!("{singular_flag}={name}") {
            return true;
        }
    }
    false
}

fn target_kind_plural_flag(kind: &str) -> &'static str {
    match kind {
        "bin" => "--bins",
        "example" => "--examples",
        "test" => "--tests",
        "bench" => "--benches",
        _ => "--all-targets",
    }
}

fn refresh_generated_lockfile_for_locked_validation(
    options: &CliOptions,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    match refresh_generated_lockfile_for_output(options, &options.output_root) {
        Ok(Some(lockfile_path)) => {
            println!("lockfile: generated Cargo.lock reconciled before locked validation");
            validation.gates.push(ValidationGateReport {
                name: "lockfile".to_string(),
                status: "passed".to_string(),
                reason: "generated Cargo.lock was reconciled before locked validation".to_string(),
                report_path: Some(lockfile_path),
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(error) => {
            let reason = error.to_string();
            validation.gates.push(ValidationGateReport {
                name: "lockfile".to_string(),
                status: "failed".to_string(),
                reason: reason.clone(),
                report_path: Some(options.output_root.join("Cargo.lock")),
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            Err(reason.into())
        }
    }
}

fn refresh_generated_lockfile_for_output(
    options: &CliOptions,
    output_root: &Path,
) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if !locked_validation_requested(&options.cargo_check_args)
        || !options.workspace_root.join("Cargo.lock").exists()
    {
        return Ok(None);
    }

    let lockfile_path = output_root.join("Cargo.lock");
    if !lockfile_path.exists() {
        return Err("generated Cargo.lock is missing before locked validation".into());
    }

    let manifest_path = absolute_path(&output_root.join("Cargo.toml"))?;
    let working_dir = manifest_working_dir(&manifest_path);
    let offline_requested = options
        .cargo_check_args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--offline" | "--frozen"));
    let mut output = run_generate_lockfile(&manifest_path, &working_dir, true)?;
    if !output.status.success() && !offline_requested {
        output = run_generate_lockfile(&manifest_path, &working_dir, false)?;
    }
    if !output.status.success() {
        return Err(format!(
            "generated Cargo.lock could not be reconciled before locked validation\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(Some(lockfile_path))
}

fn run_generate_lockfile(
    manifest_path: &Path,
    working_dir: &Path,
    offline: bool,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut command = Command::new("cargo");
    command
        .arg("generate-lockfile")
        .arg("--manifest-path")
        .arg(manifest_path)
        .current_dir(working_dir);
    if offline {
        command.arg("--offline");
    }
    Ok(command.output()?)
}

fn locked_validation_requested(cargo_check_args: &[String]) -> bool {
    cargo_check_args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--locked" | "--frozen"))
}

fn uncovered_target_required_features(
    target: &GeneratedTargetReport,
    cargo_args: &[String],
) -> Vec<String> {
    if target.required_features.is_empty() || cargo_args.iter().any(|arg| arg == "--all-features") {
        return Vec::new();
    }

    let explicitly_enabled = enabled_cargo_features(cargo_args);
    let default_enabled = !cargo_args.iter().any(|arg| arg == "--no-default-features");
    target
        .required_features
        .iter()
        .filter(|feature| {
            !feature_is_enabled_for_target(
                feature,
                &target.package,
                &explicitly_enabled,
                default_enabled.then_some(target.default_features.as_slice()),
            )
        })
        .cloned()
        .collect()
}

fn enabled_cargo_features(cargo_args: &[String]) -> BTreeSet<String> {
    let mut features = BTreeSet::new();
    let mut index = 0;
    while index < cargo_args.len() {
        let arg = &cargo_args[index];
        if arg == "--features" {
            if let Some(value) = cargo_args.get(index + 1) {
                collect_feature_arg(value, &mut features);
                index += 2;
                continue;
            }
        } else if let Some(value) = arg.strip_prefix("--features=") {
            collect_feature_arg(value, &mut features);
        }
        index += 1;
    }
    features
}

fn collect_feature_arg(value: &str, features: &mut BTreeSet<String>) {
    for feature in value.split([',', ' ']).map(str::trim) {
        if !feature.is_empty() {
            features.insert(feature.to_string());
        }
    }
}

fn feature_is_enabled_for_target(
    feature: &str,
    package: &str,
    explicitly_enabled: &BTreeSet<String>,
    default_features: Option<&[String]>,
) -> bool {
    explicitly_enabled.contains(feature)
        || explicitly_enabled.contains(&format!("{package}/{feature}"))
        || default_features
            .is_some_and(|features| features.iter().any(|enabled| enabled == feature))
}

fn finish_validation(
    options: &CliOptions,
    report: &mut ValidationReport,
    status: &str,
    reason: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    report.status = status.to_string();
    report.reason = reason.map(str::to_string);
    if let Some(path) = validation_report_path(options) {
        write_validation_report(report, &path)?;
        println!("validation report: {}", path.display());
    }
    let failed_gates: Vec<_> = report
        .gates
        .iter()
        .filter(|gate| gate.status == "failed")
        .collect();
    let non_passed_gates: Vec<_> = report
        .gates
        .iter()
        .filter(|gate| gate.status != "passed")
        .collect();
    write_event_log(
        options,
        "validation",
        status,
        "finish validation gates",
        reason.unwrap_or("validation completed"),
        event_fields(&[
            ("gates", serde_json::json!(report.gates.len())),
            ("attempts", serde_json::json!(report.attempts.len())),
            ("failed_gates", serde_json::json!(failed_gates.len())),
            (
                "failed_gate_names",
                serde_json::json!(failed_gates
                    .iter()
                    .map(|gate| gate.name.as_str())
                    .collect::<Vec<_>>()),
            ),
            (
                "failed_gate_reasons",
                serde_json::json!(failed_gates
                    .iter()
                    .map(|gate| format!("{}: {}", gate.name, gate.reason))
                    .collect::<Vec<_>>()),
            ),
            (
                "non_passed_gates",
                serde_json::json!(non_passed_gates.len()),
            ),
            (
                "non_passed_gate_names",
                serde_json::json!(non_passed_gates
                    .iter()
                    .map(|gate| gate.name.as_str())
                    .collect::<Vec<_>>()),
            ),
            (
                "gate_outcomes",
                serde_json::json!(report
                    .gates
                    .iter()
                    .map(gate_event_summary)
                    .collect::<Vec<_>>()),
            ),
            (
                "validation_report",
                serde_json::json!(validation_report_path(options)),
            ),
        ]),
    )?;
    Ok(())
}

fn gate_event_summary(gate: &ValidationGateReport) -> BTreeMap<String, serde_json::Value> {
    event_fields(&[
        ("name", serde_json::json!(&gate.name)),
        ("status", serde_json::json!(&gate.status)),
        ("reason", serde_json::json!(&gate.reason)),
        ("report_path", serde_json::json!(&gate.report_path)),
        ("error_count", serde_json::json!(gate.error_count)),
        ("warning_count", serde_json::json!(gate.warning_count)),
        (
            "semantic_warning_hazards",
            serde_json::json!(gate.semantic_warning_hazards),
        ),
        (
            "review_warning_hazards",
            serde_json::json!(gate.review_warning_hazards),
        ),
    ])
}

fn write_validation_report(
    report: &ValidationReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn finish_validation_with_decision_log(
    options: &CliOptions,
    validation: &mut ValidationReport,
    report: &GenerateReport,
    status: &str,
    reason: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    write_feedback_diagnostics_event(options, validation)?;
    finish_validation(options, validation, status, reason)?;
    write_decision_log(options, report, Some(validation), status, reason)
}

fn write_feedback_diagnostics_event(
    options: &CliOptions,
    validation: &ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = decision_log_feedback_diagnostics(options, validation, 10);
    if summary.reports == 0 && summary.unreadable_reports == 0 {
        return Ok(());
    }
    let mut fields = feedback_diagnostic_metrics(&summary);
    fields.insert("evidence".to_string(), serde_json::json!(summary.evidence));
    write_event_log(
        options,
        "feedback_diagnostics",
        &feedback_diagnostic_status(&summary),
        "classify compiler diagnostics for live slice mining",
        "the event stream mirrors the final decision-log diagnostic buckets so parallel runners can triage failures without opening the full report first",
        fields,
    )
}

#[derive(Debug, Serialize)]
struct EventLogEntry {
    timestamp_ms: u64,
    elapsed_ms: u64,
    pid: u32,
    event: String,
    status: String,
    decision: String,
    reason: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    fields: BTreeMap<String, serde_json::Value>,
}

struct EventHeartbeat {
    stop: Option<mpsc::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl EventHeartbeat {
    fn inactive() -> Self {
        Self {
            stop: None,
            handle: None,
        }
    }
}

impl Drop for EventHeartbeat {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn initialize_event_log(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = event_log_path(options) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, "")?;
    write_event_log(
        options,
        "event_log",
        "initialized",
        "start append-only CLI event logging",
        "each line records one runtime decision so parallel runs can be inspected independently",
        event_fields(&[("path", serde_json::json!(path))]),
    )
}

fn initialize_event_log_if_absent(options: &CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = event_log_path(options) else {
        return Ok(());
    };
    if path.exists() {
        return Ok(());
    }
    initialize_event_log(options)
}

fn start_event_heartbeat(
    options: &CliOptions,
    event: &str,
    decision: &str,
    reason: &str,
    fields: BTreeMap<String, serde_json::Value>,
) -> EventHeartbeat {
    let Some(path) = event_log_path(options) else {
        return EventHeartbeat::inactive();
    };
    let event = event.to_string();
    let decision = decision.to_string();
    let reason = reason.to_string();
    let (stop, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut heartbeat = 0usize;
        loop {
            match rx.recv_timeout(Duration::from_secs(15)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    heartbeat += 1;
                    let mut heartbeat_fields = fields.clone();
                    heartbeat_fields.insert("heartbeat".to_string(), serde_json::json!(heartbeat));
                    heartbeat_fields.insert(
                        "heartbeat_interval_ms".to_string(),
                        serde_json::json!(15_000),
                    );
                    let _ = append_event_log_path(
                        &path,
                        &event,
                        "running",
                        &decision,
                        &reason,
                        heartbeat_fields,
                    );
                }
            }
        }
    });
    EventHeartbeat {
        stop: Some(stop),
        handle: Some(handle),
    }
}

struct BatchRootEventHeartbeat {
    _batch: EventHeartbeat,
    _root: EventHeartbeat,
}

fn start_batch_root_event_heartbeat(
    batch_options: &CliOptions,
    root_options: &CliOptions,
    root: &RootId,
    output_root: &Path,
    event: &str,
    decision: &str,
    reason: &str,
    fields: BTreeMap<String, serde_json::Value>,
) -> BatchRootEventHeartbeat {
    let fields = batch_root_event_fields(batch_options, root_options, root, output_root, fields);
    BatchRootEventHeartbeat {
        _batch: start_event_heartbeat(batch_options, event, decision, reason, fields.clone()),
        _root: start_event_heartbeat(root_options, event, decision, reason, fields),
    }
}

fn write_event_log(
    options: &CliOptions,
    event: &str,
    status: &str,
    decision: &str,
    reason: &str,
    fields: BTreeMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = event_log_path(options) else {
        return Ok(());
    };
    append_event_log_path(&path, event, status, decision, reason, fields)
}

fn write_batch_root_event_log(
    batch_options: &CliOptions,
    root_options: &CliOptions,
    root: &RootId,
    output_root: &Path,
    event: &str,
    status: &str,
    decision: &str,
    reason: &str,
    fields: BTreeMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let fields = batch_root_event_fields(batch_options, root_options, root, output_root, fields);
    write_event_log(
        batch_options,
        event,
        status,
        decision,
        reason,
        fields.clone(),
    )?;
    write_event_log(root_options, event, status, decision, reason, fields)
}

fn batch_root_event_fields(
    batch_options: &CliOptions,
    root_options: &CliOptions,
    root: &RootId,
    output_root: &Path,
    mut fields: BTreeMap<String, serde_json::Value>,
) -> BTreeMap<String, serde_json::Value> {
    fields
        .entry("root".to_string())
        .or_insert_with(|| serde_json::json!(root.to_string()));
    fields
        .entry("root_output_root".to_string())
        .or_insert_with(|| serde_json::json!(output_root));
    fields
        .entry("batch_output_root".to_string())
        .or_insert_with(|| serde_json::json!(&batch_options.output_root));
    fields
        .entry("root_event_log".to_string())
        .or_insert_with(|| serde_json::json!(event_log_path(root_options)));
    fields
        .entry("batch_event_log".to_string())
        .or_insert_with(|| serde_json::json!(event_log_path(batch_options)));
    fields
}

fn append_event_log_path(
    path: &Path,
    event: &str,
    status: &str,
    decision: &str,
    reason: &str,
    fields: BTreeMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let entry = EventLogEntry {
        timestamp_ms: now_unix_ms(),
        elapsed_ms: run_elapsed_ms(),
        pid: std::process::id(),
        event: event.to_string(),
        status: status.to_string(),
        decision: decision.to_string(),
        reason: reason.to_string(),
        fields,
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(&entry)?)?;
    Ok(())
}

fn event_fields(pairs: &[(&str, serde_json::Value)]) -> BTreeMap<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.clone()))
        .collect()
}

fn load_progress_event_fields(
    progress: &GenerateSessionLoadProgress,
) -> BTreeMap<String, serde_json::Value> {
    progress
        .fields
        .iter()
        .map(|(key, value)| (key.clone(), serde_json::json!(value)))
        .collect()
}

fn session_load_fields(session: &GenerateSession) -> BTreeMap<String, serde_json::Value> {
    event_fields(&[
        ("manifest_ms", serde_json::json!(session.manifest_ms())),
        ("parse_ms", serde_json::json!(session.parse_ms())),
        ("analyzer_ms", serde_json::json!(session.analyzer_ms())),
        (
            "indexed_packages",
            serde_json::json!(session.indexed_package_names()),
        ),
        (
            "indexed_source_files",
            serde_json::json!(session.indexed_source_files()),
        ),
        (
            "indexed_callables",
            serde_json::json!(session.indexed_callables()),
        ),
        ("indexed_items", serde_json::json!(session.indexed_items())),
    ])
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn run_elapsed_ms() -> u64 {
    static RUN_STARTED: OnceLock<Instant> = OnceLock::new();
    elapsed_ms(*RUN_STARTED.get_or_init(Instant::now))
}

#[derive(Debug, Serialize)]
struct DecisionLogReport {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    workspace_root: PathBuf,
    output_root: PathBuf,
    analyzer_mode: String,
    production_preset: bool,
    run_check: bool,
    run_preflight: bool,
    feedback_iterations: usize,
    feedback_repair_iterations: usize,
    deny_warnings: bool,
    root_selectors: Vec<String>,
    cargo_check_args: Vec<String>,
    reports: DecisionLogReportPaths,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation: Option<DecisionLogValidationSummary>,
    steps: Vec<DecisionLogStep>,
}

#[derive(Debug, Serialize)]
struct DecisionLogReportPaths {
    decision_log: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    slice_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline_target_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_log: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preflight_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_report: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_target_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct DecisionLogValidationSummary {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    gates: usize,
    attempts: usize,
    failed_gates: Vec<String>,
    non_passed_gates: Vec<String>,
    gate_outcomes: Vec<DecisionLogGateOutcome>,
    attempt_outcomes: Vec<DecisionLogAttemptOutcome>,
}

#[derive(Debug, Serialize)]
struct DecisionLogGateOutcome {
    name: String,
    status: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning_count: Option<usize>,
}

#[derive(Debug, Serialize)]
struct DecisionLogAttemptOutcome {
    stage: String,
    attempt: usize,
    status: String,
    reason: String,
    report_path: PathBuf,
    cargo_success: bool,
    timed_out: bool,
    error_count: usize,
    warning_count: usize,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
    widening_candidates: usize,
    widening_hazards: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_widened_roots: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_report_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_total_changes: Option<usize>,
}

#[derive(Debug, Serialize)]
struct DecisionLogStep {
    step: String,
    status: String,
    decision: String,
    reason: String,
    metrics: BTreeMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    evidence: Vec<String>,
}

#[derive(Debug, Default)]
struct FeedbackDiagnosticLogSummary {
    reports: usize,
    unreadable_reports: usize,
    diagnostics: usize,
    errors: usize,
    warnings: usize,
    widening_candidates: usize,
    widening_hazards: usize,
    resolution_reports: usize,
    resolution_errors: usize,
    resolution_matched_roots: usize,
    resolution_skipped_no_match: usize,
    resolution_skipped_too_many_matches: usize,
    source_api_mismatch_candidates: usize,
    glob_import_context_entries: usize,
    glob_import_missing_export_candidates: usize,
    glob_import_private_upstream_candidates: usize,
    feedback_baseline_compared: Option<bool>,
    feedback_baseline_known_errors: Option<usize>,
    feedback_baseline_new_errors: Option<usize>,
    evidence: Vec<String>,
}

fn write_decision_log(
    options: &CliOptions,
    report: &GenerateReport,
    validation: Option<&ValidationReport>,
    status: &str,
    reason: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = decision_log_path(options) else {
        return Ok(());
    };
    let log = build_decision_log(options, report, validation, &path, status, reason);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&log)?)?;
    println!("decision log: {}", path.display());
    Ok(())
}

fn build_decision_log(
    options: &CliOptions,
    report: &GenerateReport,
    validation: Option<&ValidationReport>,
    decision_log_path: &Path,
    status: &str,
    reason: Option<&str>,
) -> DecisionLogReport {
    let validation_summary = validation.map(|validation| DecisionLogValidationSummary {
        status: validation.status.clone(),
        reason: validation.reason.clone(),
        gates: validation.gates.len(),
        attempts: validation.attempts.len(),
        failed_gates: validation
            .gates
            .iter()
            .filter(|gate| gate.status == "failed")
            .map(|gate| gate.name.clone())
            .collect(),
        non_passed_gates: validation
            .gates
            .iter()
            .filter(|gate| gate.status != "passed")
            .map(|gate| gate.name.clone())
            .collect(),
        gate_outcomes: validation
            .gates
            .iter()
            .map(|gate| DecisionLogGateOutcome {
                name: gate.name.clone(),
                status: gate.status.clone(),
                reason: gate.reason.clone(),
                report_path: gate.report_path.clone(),
                error_count: gate.error_count,
                warning_count: gate.warning_count,
            })
            .collect(),
        attempt_outcomes: validation
            .attempts
            .iter()
            .map(|attempt| DecisionLogAttemptOutcome {
                stage: attempt.stage.clone(),
                attempt: attempt.attempt,
                status: attempt.status.clone(),
                reason: attempt.reason.clone(),
                report_path: attempt.report_path.clone(),
                cargo_success: attempt.cargo_success,
                timed_out: attempt.timed_out,
                error_count: attempt.error_count,
                warning_count: attempt.warning_count,
                semantic_warning_hazards: attempt.semantic_warning_hazards,
                repairable_warnings: attempt.repairable_warnings,
                widening_candidates: attempt.widening_candidates,
                widening_hazards: attempt.widening_hazards,
                feedback_widened_roots: attempt.feedback_widened_roots,
                repair_report_path: attempt.repair_report_path.clone(),
                repair_total_changes: attempt.repair_total_changes,
            })
            .collect(),
    });
    DecisionLogReport {
        status: status.to_string(),
        reason: reason.map(str::to_string),
        workspace_root: options.workspace_root.clone(),
        output_root: options.output_root.clone(),
        analyzer_mode: options.analyzer_mode.as_str().to_string(),
        production_preset: options.production_preset,
        run_check: options.run_check,
        run_preflight: options.run_preflight,
        feedback_iterations: options.feedback_iterations,
        feedback_repair_iterations: options.feedback_repair_iterations,
        deny_warnings: options.deny_warnings,
        root_selectors: options.root_selectors.clone(),
        cargo_check_args: options.cargo_check_args.clone(),
        reports: DecisionLogReportPaths {
            decision_log: decision_log_path.to_path_buf(),
            slice_report: slice_report_path(options),
            validation_report: validation_report_path(options),
            baseline_report: options
                .run_baseline_check
                .then(|| baseline_report_path(options)),
            baseline_target_dir: options
                .run_baseline_check
                .then(|| baseline_target_dir(options)),
            event_log: event_log_path(options),
            preflight_report: maybe_preflight_report_path(options),
            feedback_report: decision_log_feedback_report_path(options),
            repair_report: repair_report_path(options),
            feedback_target_dir: (options.run_check
                || options.feedback_iterations > 0
                || options.feedback_repair_iterations > 0)
                .then(|| feedback_target_dir(options)),
        },
        validation: validation_summary,
        steps: decision_log_steps(options, report, validation),
    }
}

fn decision_log_steps(
    options: &CliOptions,
    report: &GenerateReport,
    validation: Option<&ValidationReport>,
) -> Vec<DecisionLogStep> {
    let mut steps = Vec::new();

    let mut input_metrics = BTreeMap::new();
    input_metrics.insert(
        "explicit_root_selectors".to_string(),
        serde_json::json!(options.root_selectors.len()),
    );
    input_metrics.insert(
        "cargo_check_args".to_string(),
        serde_json::json!(options.cargo_check_args.len()),
    );
    steps.push(DecisionLogStep {
        step: "input".to_string(),
        status: "configured".to_string(),
        decision: if options.root_selectors.is_empty() {
            "use source markers".to_string()
        } else {
            "use explicit in-memory roots".to_string()
        },
        reason: if options.root_selectors.is_empty() {
            "no --root or --roots-file was provided, so #[opensourced] markers define the top slice surface".to_string()
        } else {
            "--root/--roots-file selects the top slice surface without modifying the source checkout".to_string()
        },
        metrics: input_metrics,
        evidence: options.root_selectors.clone(),
    });

    let mut analyzer_metrics = BTreeMap::new();
    analyzer_metrics.insert(
        "loaded".to_string(),
        serde_json::json!(report.analyzer.loaded),
    );
    analyzer_metrics.insert(
        "semantic_available".to_string(),
        serde_json::json!(report.analyzer.semantic.is_some()),
    );
    analyzer_metrics.insert(
        "semantic_hint_callable_owners".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.callable_edges.len()),
    );
    analyzer_metrics.insert(
        "semantic_hint_item_owners".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.item_edges.len()),
    );
    analyzer_metrics.insert(
        "semantic_hint_edges".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.total_edges()),
    );
    analyzer_metrics.insert(
        "semantic_unresolved_queries".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.unresolved_queries),
    );
    analyzer_metrics.insert(
        "semantic_unqueried_queries".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.unqueried_queries),
    );
    analyzer_metrics.insert(
        "semantic_unmapped_targets".to_string(),
        serde_json::json!(report.analyzer.semantic_hints.unmapped_targets),
    );
    if let Some(semantic) = &report.analyzer.semantic {
        analyzer_metrics.insert(
            "semantic_source_files".to_string(),
            serde_json::json!(semantic.source_files),
        );
        analyzer_metrics.insert(
            "semantic_analyzed_files".to_string(),
            serde_json::json!(semantic.analyzed_files),
        );
        analyzer_metrics.insert(
            "selected_root_source_files".to_string(),
            serde_json::json!(semantic.selected_root_source_files),
        );
        analyzer_metrics.insert(
            "selected_root_analyzed_files".to_string(),
            serde_json::json!(semantic.selected_root_analyzed_files),
        );
        analyzer_metrics.insert(
            "selected_root_unresolved_paths".to_string(),
            serde_json::json!(semantic.selected_root_unresolved_paths),
        );
        analyzer_metrics.insert(
            "selected_root_unresolved_method_calls".to_string(),
            serde_json::json!(semantic.selected_root_unresolved_method_calls),
        );
    }
    steps.push(DecisionLogStep {
        step: "analyzer".to_string(),
        status: if report.analyzer.semantic.is_some() {
            semantic_proof_status(report.analyzer.semantic.as_ref().unwrap())
        } else if report.analyzer.loaded {
            "loaded_without_semantics".to_string()
        } else {
            "syntactic_only".to_string()
        },
        decision: format!(
            "run {} via {}",
            report.analyzer.mode.as_str(),
            report.analyzer.engine
        ),
        reason: "the slicer uses the syntactic index plus available rust-analyzer semantic hints to expand the downstream dependency closure".to_string(),
        metrics: analyzer_metrics,
        evidence: report.analyzer.notes.clone(),
    });

    let mut root_metrics = BTreeMap::new();
    root_metrics.insert("roots".to_string(), serde_json::json!(report.roots.len()));
    root_metrics.insert(
        "packages".to_string(),
        serde_json::json!(report.packages.len()),
    );
    steps.push(DecisionLogStep {
        step: "root_selection".to_string(),
        status: "selected".to_string(),
        decision: "treat selected roots as the only top-level open-source surface".to_string(),
        reason: "reverse dependents are not retained; the retained workspace is grown only from what selected roots depend on".to_string(),
        metrics: root_metrics,
        evidence: report.roots.iter().map(ToString::to_string).collect(),
    });

    let mut closure_metrics = BTreeMap::new();
    closure_metrics.insert(
        "reachable_callables".to_string(),
        serde_json::json!(report.reachable.len()),
    );
    closure_metrics.insert(
        "reachable_items".to_string(),
        serde_json::json!(report.reachable_items.len()),
    );
    closure_metrics.insert(
        "rendered_callables".to_string(),
        serde_json::json!(rendered_symbol_count(report, "callable")),
    );
    closure_metrics.insert(
        "rendered_items".to_string(),
        serde_json::json!(rendered_symbol_count(report, "item")),
    );
    closure_metrics.insert(
        "files_written".to_string(),
        serde_json::json!(report.files_written),
    );
    closure_metrics.insert(
        "total_ms".to_string(),
        serde_json::json!(report.timings.total_ms),
    );
    closure_metrics.insert(
        "analyzer_ms".to_string(),
        serde_json::json!(report.timings.analyzer_ms),
    );
    closure_metrics.insert(
        "reduce_ms".to_string(),
        serde_json::json!(report.timings.reduce_ms),
    );
    closure_metrics.insert(
        "render_ms".to_string(),
        serde_json::json!(report.timings.render_ms),
    );
    steps.push(DecisionLogStep {
        step: "top_down_closure".to_string(),
        status: "rendered".to_string(),
        decision: "copy and render only the selected roots plus their downstream dependency closure".to_string(),
        reason: "the top-down closure is the production direction: start at the open-source surface, walk dependencies, then render the retained set".to_string(),
        metrics: closure_metrics,
        evidence: report.packages.clone(),
    });

    let mut file_metrics = BTreeMap::new();
    file_metrics.insert(
        "retained_packages".to_string(),
        serde_json::json!(report.packages.len()),
    );
    file_metrics.insert(
        "generated_targets".to_string(),
        serde_json::json!(report.targets.len()),
    );
    file_metrics.insert(
        "files_written".to_string(),
        serde_json::json!(report.files_written),
    );
    file_metrics.insert(
        "source_files_with_reachable_symbols".to_string(),
        serde_json::json!(retained_source_file_count(report)),
    );
    file_metrics.insert(
        "reachable_callables".to_string(),
        serde_json::json!(report.reachable.len()),
    );
    file_metrics.insert(
        "reachable_items".to_string(),
        serde_json::json!(report.reachable_items.len()),
    );
    let mut file_evidence = Vec::new();
    file_evidence.extend(
        report
            .packages
            .iter()
            .take(12)
            .map(|package| format!("package {package}")),
    );
    file_evidence.extend(report.targets.iter().take(12).map(|target| {
        format!(
            "target {}::{} {:?} src={}",
            target.package,
            target.name,
            target.kind,
            target.src_path.display()
        )
    }));
    file_evidence.extend(decision_log_retained_source_file_samples(report, 24));
    steps.push(DecisionLogStep {
        step: "file_retention".to_string(),
        status: "explained".to_string(),
        decision: "write package manifests, selected targets, and source files that contain retained top-down symbols".to_string(),
        reason: "a generated file exists because it is required to describe a retained package/target, to compile a source module containing selected roots or downstream dependencies, or to carry a copied asset/include surface that compiler feedback must validate".to_string(),
        metrics: file_metrics,
        evidence: file_evidence,
    });

    let package_audit = decision_log_package_surface_audit(report);
    let root_packages = report
        .roots
        .iter()
        .map(|root| root.package().to_string())
        .collect::<BTreeSet<_>>();
    let mut package_metrics = BTreeMap::new();
    package_metrics.insert(
        "root_packages".to_string(),
        serde_json::json!(root_packages.len()),
    );
    package_metrics.insert(
        "dependency_packages".to_string(),
        serde_json::json!(package_audit
            .keys()
            .filter(|package| !root_packages.contains(*package))
            .count()),
    );
    package_metrics.insert(
        "packages_with_used_surface".to_string(),
        serde_json::json!(package_audit
            .values()
            .filter(|counts| counts.used_callables + counts.used_items > 0)
            .count()),
    );
    package_metrics.insert(
        "packages_with_unknown_surface".to_string(),
        serde_json::json!(package_audit
            .values()
            .filter(|counts| counts.blocked_by_unknown_callables
                + counts.blocked_by_unknown_items
                + counts.unknown_surfaces
                > 0)
            .count()),
    );
    steps.push(DecisionLogStep {
        step: "dependency_retention".to_string(),
        status: if report.usage.summary.blocked_by_unknown_callables
            + report.usage.summary.blocked_by_unknown_items
            + report.usage.summary.unknown_surfaces
            > 0
        {
            "retained_used_and_unknown".to_string()
        } else {
            "retained_used_only".to_string()
        },
        decision: "retain downstream package surfaces only when selected roots depend on them"
            .to_string(),
        reason: "package retention is top-down: root packages are selected explicitly, dependency packages survive only when reachable callables/items, rendered public surfaces, or scoped unknown guards require them; reverse dependents do not keep code alive".to_string(),
        metrics: package_metrics,
        evidence: decision_log_package_surface_samples(&package_audit, &root_packages, 32),
    });

    let macro_summary = &report.macro_surfaces.summary;
    let mut macro_metrics = BTreeMap::new();
    macro_metrics.insert("total".to_string(), serde_json::json!(macro_summary.total));
    macro_metrics.insert(
        "derive_macros".to_string(),
        serde_json::json!(macro_summary.derive_macros),
    );
    macro_metrics.insert(
        "attribute_macros".to_string(),
        serde_json::json!(macro_summary.attribute_macros),
    );
    macro_metrics.insert(
        "helper_attributes".to_string(),
        serde_json::json!(macro_summary.helper_attributes),
    );
    macro_metrics.insert(
        "macro_invocations".to_string(),
        serde_json::json!(macro_summary.macro_invocations),
    );
    macro_metrics.insert(
        "macro_blocked".to_string(),
        serde_json::json!(macro_summary.macro_blocked),
    );
    steps.push(DecisionLogStep {
        step: "macro_surfaces".to_string(),
        status: if macro_summary.macro_blocked > 0 {
            "fail_closed".to_string()
        } else {
            "classified".to_string()
        },
        decision: "scope macro uncertainty to touched macro surfaces".to_string(),
        reason: "unknown macro expansion can require token-named helpers, so affected owners are blocked_by_unknown instead of pruned".to_string(),
        metrics: macro_metrics,
        evidence: report
            .macro_surfaces
            .surfaces
            .iter()
            .take(20)
            .map(|surface| {
                format!(
                    "{} {} on {}",
                    surface.category, surface.path, surface.subject
                )
            })
            .collect(),
    });

    let usage = &report.usage.summary;
    let rendered_usage = rendered_usage_contract(report);
    let mut usage_metrics = BTreeMap::new();
    usage_metrics.insert(
        "used_callables".to_string(),
        serde_json::json!(usage.used_callables),
    );
    usage_metrics.insert(
        "used_items".to_string(),
        serde_json::json!(usage.used_items),
    );
    usage_metrics.insert(
        "prunable_callables".to_string(),
        serde_json::json!(usage.prunable_callables),
    );
    usage_metrics.insert(
        "prunable_items".to_string(),
        serde_json::json!(usage.prunable_items),
    );
    usage_metrics.insert(
        "blocked_by_unknown_callables".to_string(),
        serde_json::json!(usage.blocked_by_unknown_callables),
    );
    usage_metrics.insert(
        "blocked_by_unknown_items".to_string(),
        serde_json::json!(usage.blocked_by_unknown_items),
    );
    usage_metrics.insert(
        "rendered_used".to_string(),
        serde_json::json!(rendered_usage.used),
    );
    usage_metrics.insert(
        "rendered_blocked_by_unknown".to_string(),
        serde_json::json!(rendered_usage.blocked_by_unknown),
    );
    usage_metrics.insert(
        "rendered_invalid".to_string(),
        serde_json::json!(rendered_usage.invalid.len()),
    );
    steps.push(DecisionLogStep {
        step: "usage_pruning".to_string(),
        status: if rendered_usage.invalid.is_empty() {
            "contract_ok".to_string()
        } else {
            "contract_failed".to_string()
        },
        decision: "strip prunable unused symbols and retain used or blocked_by_unknown symbols".to_string(),
        reason: "generated source must not retain known-unused code except where an unknown surface blocks safe deletion".to_string(),
        metrics: usage_metrics,
        evidence: decision_log_usage_samples(report, &rendered_usage, 36),
    });

    let semantic_usage_proof = &report.usage.semantic_proof;
    let semantic_usage_summary = &semantic_usage_proof.summary;
    let mut semantic_usage_metrics = BTreeMap::new();
    semantic_usage_metrics.insert(
        "analyzer_available".to_string(),
        serde_json::json!(semantic_usage_summary.analyzer_available),
    );
    semantic_usage_metrics.insert(
        "retained_packages".to_string(),
        serde_json::json!(semantic_usage_summary.retained_packages),
    );
    semantic_usage_metrics.insert(
        "prunable_callables".to_string(),
        serde_json::json!(semantic_usage_summary.prunable_callables),
    );
    semantic_usage_metrics.insert(
        "prunable_items".to_string(),
        serde_json::json!(semantic_usage_summary.prunable_items),
    );
    semantic_usage_metrics.insert(
        "proof_required_callables".to_string(),
        serde_json::json!(semantic_usage_summary.proof_required_callables),
    );
    semantic_usage_metrics.insert(
        "proof_required_items".to_string(),
        serde_json::json!(semantic_usage_summary.proof_required_items),
    );
    semantic_usage_metrics.insert(
        "proven_callables".to_string(),
        serde_json::json!(semantic_usage_summary.proven_callables),
    );
    semantic_usage_metrics.insert(
        "proven_items".to_string(),
        serde_json::json!(semantic_usage_summary.proven_items),
    );
    semantic_usage_metrics.insert(
        "unproven_callables".to_string(),
        serde_json::json!(semantic_usage_summary.unproven_callables),
    );
    semantic_usage_metrics.insert(
        "unproven_items".to_string(),
        serde_json::json!(semantic_usage_summary.unproven_items),
    );
    semantic_usage_metrics.insert(
        "cfg_inactive_callables".to_string(),
        serde_json::json!(semantic_usage_summary.cfg_inactive_callables),
    );
    semantic_usage_metrics.insert(
        "cfg_inactive_items".to_string(),
        serde_json::json!(semantic_usage_summary.cfg_inactive_items),
    );
    semantic_usage_metrics.insert(
        "source_file_pruned_callables".to_string(),
        serde_json::json!(semantic_usage_summary.source_file_pruned_callables),
    );
    semantic_usage_metrics.insert(
        "source_file_pruned_items".to_string(),
        serde_json::json!(semantic_usage_summary.source_file_pruned_items),
    );
    semantic_usage_metrics.insert(
        "structural_pruned_items".to_string(),
        serde_json::json!(semantic_usage_summary.structural_pruned_items),
    );
    semantic_usage_metrics.insert(
        "unmapped_callables".to_string(),
        serde_json::json!(semantic_usage_summary.unmapped_callables),
    );
    semantic_usage_metrics.insert(
        "unmapped_items".to_string(),
        serde_json::json!(semantic_usage_summary.unmapped_items),
    );
    semantic_usage_metrics.insert(
        "failed_reference_query_callables".to_string(),
        serde_json::json!(semantic_usage_summary.failed_reference_query_callables),
    );
    semantic_usage_metrics.insert(
        "failed_reference_query_items".to_string(),
        serde_json::json!(semantic_usage_summary.failed_reference_query_items),
    );
    semantic_usage_metrics.insert(
        "retained_reference_callables".to_string(),
        serde_json::json!(semantic_usage_summary.retained_reference_callables),
    );
    semantic_usage_metrics.insert(
        "retained_reference_items".to_string(),
        serde_json::json!(semantic_usage_summary.retained_reference_items),
    );
    steps.push(DecisionLogStep {
        step: "semantic_usage_proof".to_string(),
        status: semantic_usage_proof.status.clone(),
        decision: "accept pruning only when retained-package candidates are proven unused or explicitly discharged".to_string(),
        reason: "rust-analyzer reference results are the proof source; cfg-inactive, source-file-pruned, and structural-pruned buckets explain safe pruning when RA cannot map or query a candidate that is not rendered".to_string(),
        metrics: semantic_usage_metrics,
        evidence: decision_log_semantic_usage_proof_evidence(report, 32),
    });

    let rendered_summary = &report.usage.rendered_symbols.summary;
    let rendered_decisions = &report.usage.rendered_decision_map;
    let mut member_metrics = BTreeMap::new();
    member_metrics.insert(
        "rendered_members".to_string(),
        serde_json::json!(rendered_summary.rendered_members),
    );
    member_metrics.insert(
        "retained_members".to_string(),
        serde_json::json!(rendered_summary.retained_members),
    );
    member_metrics.insert(
        "blocked_members".to_string(),
        serde_json::json!(rendered_summary.blocked_members),
    );
    member_metrics.insert(
        "prunable_members".to_string(),
        serde_json::json!(rendered_summary.prunable_members),
    );
    member_metrics.insert(
        "unclassified_members".to_string(),
        serde_json::json!(rendered_summary.unclassified_members),
    );
    member_metrics.insert(
        "rendered_assoc_items".to_string(),
        serde_json::json!(rendered_summary.rendered_assoc_items),
    );
    member_metrics.insert(
        "retained_assoc_items".to_string(),
        serde_json::json!(rendered_summary.retained_assoc_items),
    );
    member_metrics.insert(
        "blocked_assoc_items".to_string(),
        serde_json::json!(rendered_summary.blocked_assoc_items),
    );
    member_metrics.insert(
        "prunable_assoc_items".to_string(),
        serde_json::json!(rendered_summary.prunable_assoc_items),
    );
    member_metrics.insert(
        "unclassified_assoc_items".to_string(),
        serde_json::json!(rendered_summary.unclassified_assoc_items),
    );
    member_metrics.insert(
        "member_decisions".to_string(),
        serde_json::json!(rendered_decisions.members.len()),
    );
    member_metrics.insert(
        "assoc_item_decisions".to_string(),
        serde_json::json!(rendered_decisions.assoc_items.len()),
    );
    let mut member_evidence =
        decision_log_decision_samples("member", &rendered_decisions.members, 20);
    member_evidence.extend(decision_log_decision_samples(
        "assoc_item",
        &rendered_decisions.assoc_items,
        20,
    ));
    steps.push(DecisionLogStep {
        step: "member_pruning".to_string(),
        status: if rendered_summary.prunable_members > 0
            || rendered_summary.prunable_assoc_items > 0
            || rendered_summary.unclassified_members > 0
            || rendered_summary.unclassified_assoc_items > 0
        {
            "review_required".to_string()
        } else if rendered_summary.blocked_members > 0 || rendered_summary.blocked_assoc_items > 0 {
            "retained_unknown_surfaces".to_string()
        } else {
            "exact_used_surface".to_string()
        },
        decision: "retain only member and assoc-item surfaces classified as used or blocked_by_unknown"
            .to_string(),
        reason: "member pruning follows top-down concrete uses from retained bodies, macro-token field reads, public/root surfaces, and unknown-surface guards; known-unused members must not survive in generated root or dependency packages".to_string(),
        metrics: member_metrics,
        evidence: member_evidence,
    });

    let mut import_metrics = BTreeMap::new();
    import_metrics.insert(
        "retained_members".to_string(),
        serde_json::json!(rendered_summary.retained_members),
    );
    import_metrics.insert(
        "blocked_members".to_string(),
        serde_json::json!(rendered_summary.blocked_members),
    );
    import_metrics.insert(
        "prunable_members".to_string(),
        serde_json::json!(rendered_summary.prunable_members),
    );
    import_metrics.insert(
        "retained_assoc_items".to_string(),
        serde_json::json!(rendered_summary.retained_assoc_items),
    );
    import_metrics.insert(
        "prunable_assoc_items".to_string(),
        serde_json::json!(rendered_summary.prunable_assoc_items),
    );
    steps.push(DecisionLogStep {
        step: "import_pruning".to_string(),
        status: if rendered_usage.blocked_by_unknown > 0 {
            "retained_unknown_surfaces".to_string()
        } else {
            "exact_used_surface".to_string()
        },
        decision: "retain imports only when transformed callable bodies or retained non-callable surfaces still use them".to_string(),
        reason: "import pruning follows the same top-down retained surface as rendering, so fields and struct literal entries pruned from the output cannot keep stale imports alive".to_string(),
        metrics: import_metrics,
        evidence: Vec::new(),
    });

    let mut readiness_metrics = BTreeMap::new();
    readiness_metrics.insert(
        "hazards".to_string(),
        serde_json::json!(report.production.hazards.len()),
    );
    readiness_metrics.insert(
        "error_hazards".to_string(),
        serde_json::json!(report
            .production
            .hazards
            .iter()
            .filter(|hazard| hazard.severity == "error")
            .count()),
    );
    readiness_metrics.insert(
        "warning_hazards".to_string(),
        serde_json::json!(report
            .production
            .hazards
            .iter()
            .filter(|hazard| hazard.severity == "warning")
            .count()),
    );
    steps.push(DecisionLogStep {
        step: "production_readiness".to_string(),
        status: report.production.status.clone(),
        decision: "accept only if hard validation gates pass".to_string(),
        reason: "hazards describe remaining semantic uncertainty; error hazards fail production validation unless specifically discharged".to_string(),
        metrics: readiness_metrics,
        evidence: report
            .production
            .hazards
            .iter()
            .take(20)
            .map(|hazard| format!("{} {}: {}", hazard.severity, hazard.code, hazard.message))
            .collect(),
    });

    if let Some(validation) = validation {
        if !validation.attempts.is_empty() {
            let last_attempt = validation.attempts.last();
            let mut feedback_metrics = BTreeMap::new();
            feedback_metrics.insert(
                "attempts".to_string(),
                serde_json::json!(validation.attempts.len()),
            );
            feedback_metrics.insert(
                "accepted_attempts".to_string(),
                serde_json::json!(validation
                    .attempts
                    .iter()
                    .filter(|attempt| attempt.status == "accepted")
                    .count()),
            );
            feedback_metrics.insert(
                "repaired_attempts".to_string(),
                serde_json::json!(validation
                    .attempts
                    .iter()
                    .filter(|attempt| attempt.status == "repaired")
                    .count()),
            );
            feedback_metrics.insert(
                "widened_attempts".to_string(),
                serde_json::json!(validation
                    .attempts
                    .iter()
                    .filter(|attempt| attempt.status == "widened")
                    .count()),
            );
            feedback_metrics.insert(
                "rejected_attempts".to_string(),
                serde_json::json!(validation
                    .attempts
                    .iter()
                    .filter(|attempt| matches!(
                        attempt.status.as_str(),
                        "rejected" | "failed" | "timed_out" | "no_progress" | "low_progress"
                    ))
                    .count()),
            );
            feedback_metrics.insert(
                "repair_total_changes".to_string(),
                serde_json::json!(validation
                    .attempts
                    .iter()
                    .filter_map(|attempt| attempt.repair_total_changes)
                    .sum::<usize>()),
            );
            feedback_metrics.insert(
                "last_errors".to_string(),
                serde_json::json!(last_attempt.map(|attempt| attempt.error_count)),
            );
            feedback_metrics.insert(
                "last_warnings".to_string(),
                serde_json::json!(last_attempt.map(|attempt| attempt.warning_count)),
            );
            steps.push(DecisionLogStep {
                step: "compiler_feedback".to_string(),
                status: last_attempt
                    .map(|attempt| attempt.status.clone())
                    .unwrap_or_else(|| "not_run".to_string()),
                decision: "check generated workspace and choose accept, conservative repair, root widening, or rejection".to_string(),
                reason: "compiler feedback is the final runtime oracle for generated syntax, imports, lint hazards, and repair progress".to_string(),
                metrics: feedback_metrics,
                evidence: validation
                    .attempts
                    .iter()
                    .map(|attempt| {
                        format!(
                            "{}#{} {}: {} (errors={}, warnings={}, repair_changes={})",
                            attempt.stage,
                            attempt.attempt,
                            attempt.status,
                            attempt.reason,
                            attempt.error_count,
                            attempt.warning_count,
                            attempt.repair_total_changes.unwrap_or(0)
                        )
                    })
                    .collect(),
            });
        }

        let diagnostic_summary = decision_log_feedback_diagnostics(options, validation, 40);
        if diagnostic_summary.reports > 0 || diagnostic_summary.unreadable_reports > 0 {
            steps.push(DecisionLogStep {
                step: "feedback_diagnostics".to_string(),
                status: feedback_diagnostic_status(&diagnostic_summary),
                decision: "record compiler diagnostic shape for slice mining and failure triage"
                    .to_string(),
                reason: "diagnostic codes, primary files, widening candidates, and warning/error counts explain why a generated slice was accepted, repaired, widened, or rejected without opening the raw feedback JSON first".to_string(),
                metrics: feedback_diagnostic_metrics(&diagnostic_summary),
                evidence: diagnostic_summary.evidence,
            });
        }

        let mut validation_metrics = BTreeMap::new();
        validation_metrics.insert(
            "gates".to_string(),
            serde_json::json!(validation.gates.len()),
        );
        validation_metrics.insert(
            "attempts".to_string(),
            serde_json::json!(validation.attempts.len()),
        );
        validation_metrics.insert(
            "failed_gates".to_string(),
            serde_json::json!(validation
                .gates
                .iter()
                .filter(|gate| gate.status == "failed")
                .count()),
        );
        steps.push(DecisionLogStep {
            step: "validation".to_string(),
            status: validation.status.clone(),
            decision: "use validation gates as the acceptance contract".to_string(),
            reason: validation
                .reason
                .clone()
                .unwrap_or_else(|| "all configured validation gates completed".to_string()),
            metrics: validation_metrics,
            evidence: validation
                .gates
                .iter()
                .map(|gate| format!("{}={}: {}", gate.name, gate.status, gate.reason))
                .collect(),
        });
    }

    steps
}

fn rendered_symbol_count(report: &GenerateReport, kind: &str) -> usize {
    report
        .usage
        .rendered_symbols
        .entries
        .iter()
        .filter(|entry| entry.kind == kind)
        .count()
}

#[allow(clippy::too_many_arguments)]
fn record_feedback_attempt(
    validation: &mut ValidationReport,
    stage: &str,
    attempt: usize,
    status: &str,
    reason: &str,
    report: &CheckReport,
    report_path: PathBuf,
    baseline_limited: bool,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
    repair_report_path: Option<PathBuf>,
    repair_total_changes: Option<usize>,
) {
    validation.attempts.push(ValidationAttemptReport {
        stage: stage.to_string(),
        attempt,
        status: status.to_string(),
        reason: reason.to_string(),
        report_path,
        cargo_success: report.success,
        baseline_limited,
        timed_out: report.timed_out,
        error_count: report.error_count(),
        warning_count: report.warning_count(),
        semantic_warning_hazards,
        repairable_warnings,
        widening_candidates: report.widening.candidates.len(),
        widening_hazards: report.widening.hazards.len(),
        feedback_widened_roots: None,
        repair_report_path,
        repair_total_changes,
    });
}

fn record_feedback_gate(
    validation: &mut ValidationReport,
    name: &str,
    status: &str,
    reason: &str,
    report: &CheckReport,
    report_path: PathBuf,
    semantic_warning_hazards: usize,
) {
    validation.add_check_gate(
        name,
        status,
        reason,
        report,
        Some(report_path),
        semantic_warning_hazards,
    );
}

fn record_production_readiness_gate(
    options: &CliOptions,
    validation: &mut ValidationReport,
    production: &opensource_core::ProductionReadinessReport,
    phase: &str,
) {
    let status = production_readiness_gate_status(options, production);
    let error_count = production
        .hazards
        .iter()
        .filter(|hazard| hazard.severity == "error")
        .count();
    let warning_count = production
        .hazards
        .iter()
        .filter(|hazard| hazard.severity == "warning")
        .count();
    let discharged_error_count = production
        .hazards
        .iter()
        .filter(|hazard| {
            hazard.severity == "error" && production_error_hazard_is_discharged(options, hazard)
        })
        .count();
    let reason = if discharged_error_count > 0 {
        format!(
            "{} production hazard(s) reported {phase}; {} feature cfg error hazard(s) covered by validation cargo arguments",
            production.hazards.len(),
            discharged_error_count
        )
    } else {
        format!(
            "{} production hazard(s) reported {phase}",
            production.hazards.len()
        )
    };
    validation.gates.push(ValidationGateReport {
        name: "production_readiness".to_string(),
        status,
        reason,
        report_path: slice_report_path(options),
        error_count: Some(error_count),
        warning_count: Some(warning_count),
        semantic_warning_hazards: None,
        review_warning_hazards: Some(review_warning_hazard_count(production)),
    });
}

#[allow(clippy::too_many_arguments)]
fn try_widen_from_feedback(
    options: &CliOptions,
    validation: &mut ValidationReport,
    state: &mut FeedbackWideningState,
    stage: &str,
    attempt: usize,
    report: &CheckReport,
    report_path: &Path,
    semantic_warning_hazards: usize,
    repairable_warnings: usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    if report.error_count() == 0 {
        return Ok(false);
    }

    state.diagnostics.extend(report.diagnostics.clone());
    write_event_log(
        options,
        "feedback_widening",
        "started",
        "rerender generated workspace from compiler feedback roots",
        "missing symbols and unresolved paths are converted into additional top-down roots before retrying cargo feedback",
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("attempt", serde_json::json!(attempt)),
            ("diagnostics", serde_json::json!(state.diagnostics.len())),
            (
                "widening_candidates",
                serde_json::json!(report.widening.candidates.len()),
            ),
            (
                "widening_hazards",
                serde_json::json!(report.widening.hazards.len()),
            ),
            ("report_path", serde_json::json!(report_path)),
        ]),
    )?;
    let resolution = resolve_feedback_widening_roots(
        &options.workspace_root,
        &state.diagnostics,
        &options.root_selectors,
    )?;
    write_event_log(
        options,
        "feedback_widening_resolution",
        "completed",
        "resolve compiler feedback symbols to top-down roots before rerender",
        "the slicer only pays the full analyzer/render cost when diagnostics map to a new bounded root set",
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("attempt", serde_json::json!(attempt)),
            ("diagnostics", serde_json::json!(resolution.diagnostics)),
            (
                "error_diagnostics",
                serde_json::json!(resolution.error_diagnostics),
            ),
            (
                "candidate_symbols",
                serde_json::json!(resolution.candidate_symbols),
            ),
            (
                "matched_roots",
                serde_json::json!(resolution.matched_roots.len()),
            ),
            (
                "skipped_no_match",
                serde_json::json!(resolution.skipped_no_match),
            ),
            (
                "skipped_too_many_matches",
                serde_json::json!(resolution.skipped_too_many_matches),
            ),
            (
                "skipped_marked_roots",
                serde_json::json!(resolution.skipped_marked_roots),
            ),
            (
                "entries_truncated",
                serde_json::json!(resolution.entries_truncated),
            ),
            ("manifest_ms", serde_json::json!(resolution.manifest_ms)),
            ("parse_ms", serde_json::json!(resolution.parse_ms)),
            (
                "entry_sample",
                serde_json::json!(resolution.entries.iter().take(12).collect::<Vec<_>>()),
            ),
        ]),
    )?;
    let widened_roots = resolution.matched_roots.clone();
    let widened_signature = widened_roots.join("\n");
    let widened_signature_is_empty = widened_signature.is_empty();
    if widened_signature_is_empty || !state.seen_root_sets.insert(widened_signature) {
        write_event_log(
            options,
            "feedback_widening",
            "skipped",
            "stop feedback widening without rerun",
            if widened_signature_is_empty {
                "compiler feedback produced no additional roots that can be safely widened"
            } else {
                "compiler feedback produced the same widened root set as an earlier attempt"
            },
            event_fields(&[
                ("stage", serde_json::json!(stage)),
                ("attempt", serde_json::json!(attempt)),
                ("widened_roots", serde_json::json!(widened_roots)),
                (
                    "skipped_no_match",
                    serde_json::json!(resolution.skipped_no_match),
                ),
                (
                    "skipped_too_many_matches",
                    serde_json::json!(resolution.skipped_too_many_matches),
                ),
            ]),
        )?;
        return Ok(false);
    }

    let _widening_heartbeat = start_event_heartbeat(
        options,
        "feedback_widening",
        "rerender generated workspace from compiler feedback roots",
        "feedback widening resolved new roots and is now reloading analyzer state and writing the next candidate workspace",
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("attempt", serde_json::json!(attempt)),
            ("diagnostics", serde_json::json!(state.diagnostics.len())),
            (
                "widening_candidates",
                serde_json::json!(report.widening.candidates.len()),
            ),
            (
                "widening_hazards",
                serde_json::json!(report.widening.hazards.len()),
            ),
            ("widened_roots", serde_json::json!(widened_roots.len())),
        ]),
    );
    let widened_report = generate_with_analyzer_feedback_and_roots(
        GenerateOptions {
            workspace_root: options.workspace_root.clone(),
            output_root: options.output_root.clone(),
        },
        options.analyzer_mode,
        &state.diagnostics,
        &options.root_selectors,
    )?;
    let widened_roots = widened_report
        .feedback_widened_roots
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if let Some(report_path) = slice_report_path(options) {
        write_generate_report(&widened_report, &report_path)?;
    }
    write_report(report, report_path)?;
    println!(
        "feedback: widened {} root(s) from compiler diagnostics and re-rendered generated workspace",
        widened_roots.len()
    );
    write_event_log(
        options,
        "feedback_widening",
        "completed",
        "rerender generated workspace from compiler feedback roots",
        "compiler feedback widened the top-down root set and wrote the next candidate workspace",
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("attempt", serde_json::json!(attempt)),
            ("widened_roots", serde_json::json!(&widened_roots)),
            (
                "production_status",
                serde_json::json!(&widened_report.production.status),
            ),
            (
                "production_hazards",
                serde_json::json!(widened_report.production.hazards.len()),
            ),
            (
                "files_written",
                serde_json::json!(widened_report.files_written),
            ),
        ]),
    )?;
    record_feedback_attempt(
        validation,
        stage,
        attempt,
        "widened",
        "compiler feedback widened the generated workspace",
        report,
        report_path.to_path_buf(),
        false,
        semantic_warning_hazards,
        repairable_warnings,
        None,
        None,
    );
    if let Some(recorded) = validation.attempts.last_mut() {
        recorded.feedback_widened_roots = Some(widened_roots.len());
    }
    record_production_readiness_gate(
        options,
        validation,
        &widened_report.production,
        "after compiler feedback widening",
    );
    if production_readiness_blocks_validation(options, &widened_report.production) {
        return Err(
            "production readiness reported error hazards after compiler feedback widening".into(),
        );
    }

    refresh_generated_lockfile_for_locked_validation(options, validation)?;
    let preflight = run_preflight(options)?;
    if !preflight.success {
        return Err("feedback widening produced a structurally invalid generated workspace".into());
    }

    Ok(true)
}

#[derive(Debug, Clone, Copy)]
enum FeedbackAcceptancePolicy {
    Feedback,
    Repair,
}

#[derive(Debug, Clone, Copy)]
struct PostWidenVerificationOutcome {
    accepted: bool,
    baseline_limited: bool,
}

#[allow(clippy::too_many_arguments)]
fn run_post_widen_feedback_verification(
    options: &CliOptions,
    baseline: Option<&CheckReport>,
    validation: &mut ValidationReport,
    stage: &str,
    gate_name: &str,
    attempt: usize,
    report_path: &Path,
    cargo_args: Vec<String>,
    acceptance_policy: FeedbackAcceptancePolicy,
) -> Result<PostWidenVerificationOutcome, Box<dyn std::error::Error>> {
    println!("{stage} post-widen verification: cargo check --message-format=json");
    write_event_log(
        options,
        "feedback_widening_verification",
        "started",
        "run cargo check after compiler feedback widening",
        "a final-attempt widening must be verified immediately so reports describe the current widened workspace",
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("gate", serde_json::json!(gate_name)),
            ("attempt", serde_json::json!(attempt)),
            ("target_dir", serde_json::json!(feedback_target_dir(options))),
            ("feedback_report", serde_json::json!(report_path)),
            ("cargo_args", serde_json::json!(cargo_args)),
        ]),
    )?;
    let widened_check = check_workspace(CheckOptions {
        manifest_path: options.output_root.join("Cargo.toml"),
        target_dir: Some(feedback_target_dir(options)),
        timeout: options.feedback_timeout,
        cargo_args,
    })?;
    write_report(&widened_check, report_path)?;
    print_feedback(&widened_check, options.feedback_limit, report_path);

    let semantic_warnings = semantic_hazard_warning_count(&widened_check.diagnostics, baseline);
    let repairable_warnings = repairable_warning_count(&widened_check.diagnostics);
    write_event_log(
        options,
        "feedback_widening_verification",
        if widened_check.success {
            "checked"
        } else {
            "failed"
        },
        "classify compiler feedback after widening",
        "post-widen verification decides whether the current top-down slice is accepted, baseline-limited, or rejected",
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("gate", serde_json::json!(gate_name)),
            ("attempt", serde_json::json!(attempt)),
            ("success", serde_json::json!(widened_check.success)),
            ("errors", serde_json::json!(widened_check.error_count())),
            ("warnings", serde_json::json!(widened_check.warning_count())),
            ("semantic_warnings", serde_json::json!(semantic_warnings)),
            ("repairable_warnings", serde_json::json!(repairable_warnings)),
            ("timed_out", serde_json::json!(widened_check.timed_out)),
            ("duration_ms", serde_json::json!(widened_check.duration_ms)),
        ]),
    )?;

    let accepted = match acceptance_policy {
        FeedbackAcceptancePolicy::Feedback => {
            feedback_is_accepted(&widened_check, baseline, options.deny_warnings)
        }
        FeedbackAcceptancePolicy::Repair => {
            feedback_repair_is_accepted(&widened_check, baseline, options.deny_warnings)
        }
    };
    if accepted {
        write_event_log(
            options,
            "feedback_widening_verification",
            "accepted",
            "accept generated workspace after compiler feedback widening",
            "cargo check passed after widening the top-down roots from compiler feedback",
            event_fields(&[
                ("stage", serde_json::json!(stage)),
                ("gate", serde_json::json!(gate_name)),
                ("attempt", serde_json::json!(attempt)),
            ]),
        )?;
        record_feedback_attempt(
            validation,
            stage,
            attempt,
            "accepted",
            "generated workspace cargo check passed after compiler feedback widening",
            &widened_check,
            report_path.to_path_buf(),
            false,
            semantic_warnings,
            repairable_warnings,
            None,
            None,
        );
        record_feedback_gate(
            validation,
            gate_name,
            "accepted",
            "generated workspace cargo check passed after compiler feedback widening",
            &widened_check,
            report_path.to_path_buf(),
            semantic_warnings,
        );
        return Ok(PostWidenVerificationOutcome {
            accepted: true,
            baseline_limited: false,
        });
    }

    let baseline_limited = options.allow_baseline_failures
        && match acceptance_policy {
            FeedbackAcceptancePolicy::Feedback => baseline_limited_feedback_is_accepted(
                &widened_check,
                baseline,
                options.deny_warnings,
            ),
            FeedbackAcceptancePolicy::Repair => baseline_limited_feedback_repair_is_accepted(
                &widened_check,
                baseline,
                options.deny_warnings,
            ),
        };
    if baseline_limited {
        write_event_log(
            options,
            "feedback_widening_verification",
            "baseline_limited",
            "accept baseline-limited generated workspace after widening",
            "post-widen generated errors match the source baseline and remaining gates passed",
            event_fields(&[
                ("stage", serde_json::json!(stage)),
                ("gate", serde_json::json!(gate_name)),
                ("attempt", serde_json::json!(attempt)),
            ]),
        )?;
        record_feedback_attempt(
            validation,
            stage,
            attempt,
            "baseline_limited",
            "generated errors match the source baseline after compiler feedback widening",
            &widened_check,
            report_path.to_path_buf(),
            true,
            semantic_warnings,
            repairable_warnings,
            None,
            None,
        );
        record_feedback_gate(
            validation,
            gate_name,
            "baseline_limited",
            "generated errors match the source baseline after compiler feedback widening",
            &widened_check,
            report_path.to_path_buf(),
            semantic_warnings,
        );
        return Ok(PostWidenVerificationOutcome {
            accepted: true,
            baseline_limited: true,
        });
    }

    let reason = if widened_check.timed_out {
        "post-widen feedback cargo check timed out"
    } else {
        "final verification after compiler feedback widening did not pass"
    };
    write_event_log(
        options,
        "feedback_widening_verification",
        "rejected",
        "reject generated workspace after compiler feedback widening",
        reason,
        event_fields(&[
            ("stage", serde_json::json!(stage)),
            ("gate", serde_json::json!(gate_name)),
            ("attempt", serde_json::json!(attempt)),
        ]),
    )?;
    record_feedback_attempt(
        validation,
        stage,
        attempt,
        "rejected_after_widen",
        reason,
        &widened_check,
        report_path.to_path_buf(),
        false,
        semantic_warnings,
        repairable_warnings,
        None,
        None,
    );
    record_feedback_gate(
        validation,
        gate_name,
        "failed",
        reason,
        &widened_check,
        report_path.to_path_buf(),
        semantic_warnings,
    );
    Ok(PostWidenVerificationOutcome {
        accepted: false,
        baseline_limited: false,
    })
}

fn run_feedback_loop(
    options: &CliOptions,
    baseline: Option<&CheckReport>,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_path = options
        .feedback_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));
    let mut seen_diagnostics = std::collections::BTreeSet::new();
    let mut seen_diagnostic_shapes = std::collections::BTreeSet::new();
    let mut widening_state = FeedbackWideningState::default();

    for attempt in 1..=options.feedback_iterations {
        println!(
            "feedback attempt {attempt}/{}: cargo check --message-format=json",
            options.feedback_iterations
        );
        write_event_log(
            options,
            "feedback_attempt",
            "started",
            "run cargo check on generated workspace",
            "compiler output decides whether the top-down slice is complete or needs widening",
            event_fields(&[
                ("attempt", serde_json::json!(attempt)),
                (
                    "target_dir",
                    serde_json::json!(feedback_target_dir(options)),
                ),
                ("report_path", serde_json::json!(report_path)),
            ]),
        )?;
        let report = check_workspace(CheckOptions {
            manifest_path: options.output_root.join("Cargo.toml"),
            target_dir: Some(feedback_target_dir(options)),
            timeout: options.feedback_timeout,
            cargo_args: options.cargo_check_args.clone(),
        })?;
        write_report(&report, &report_path)?;
        print_feedback(&report, options.feedback_limit, &report_path);

        let semantic_warnings = semantic_hazard_warning_count(&report.diagnostics, baseline);
        write_event_log(
            options,
            "feedback_attempt",
            if report.success { "checked" } else { "failed" },
            "classify generated workspace compiler feedback",
            "diagnostic shape and semantic warning gates decide whether to accept, widen, retry, or stop",
            event_fields(&[
                ("attempt", serde_json::json!(attempt)),
                ("success", serde_json::json!(report.success)),
                ("errors", serde_json::json!(report.error_count())),
                ("warnings", serde_json::json!(report.warning_count())),
                ("semantic_warnings", serde_json::json!(semantic_warnings)),
                ("timed_out", serde_json::json!(report.timed_out)),
                ("duration_ms", serde_json::json!(report.duration_ms)),
            ]),
        )?;
        if feedback_is_accepted(&report, baseline, options.deny_warnings) {
            write_event_log(
                options,
                "feedback",
                "accepted",
                "accept generated workspace after compiler feedback",
                "cargo check passed all feedback gates",
                event_fields(&[("attempt", serde_json::json!(attempt))]),
            )?;
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.success && semantic_warnings > 0 {
            println!("feedback: semantic warning gate rejected {semantic_warnings} new warning(s)");
        }
        if report.success && options.deny_warnings {
            println!("feedback: warnings denied by --deny-warnings");
        }
        if options.allow_baseline_failures
            && baseline_limited_feedback_is_accepted(&report, baseline, options.deny_warnings)
        {
            println!(
                "feedback: generated errors match the source baseline; treating as baseline-limited pass"
            );
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                report_path.clone(),
                true,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.timed_out {
            write_event_log(
                options,
                "feedback",
                "timed_out",
                "stop feedback loop",
                "cargo check exceeded the configured timeout",
                event_fields(&[("attempt", serde_json::json!(attempt))]),
            )?;
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "timed_out",
                "feedback cargo check timed out",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "failed",
                "feedback cargo check timed out",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Err("feedback cargo check timed out".into());
        }

        if try_widen_from_feedback(
            options,
            validation,
            &mut widening_state,
            "feedback",
            attempt,
            &report,
            &report_path,
            semantic_warnings,
            repairable_warning_count(&report.diagnostics),
        )? {
            if attempt == options.feedback_iterations {
                let outcome = run_post_widen_feedback_verification(
                    options,
                    baseline,
                    validation,
                    "feedback",
                    "feedback",
                    attempt + 1,
                    &report_path,
                    options.cargo_check_args.clone(),
                    FeedbackAcceptancePolicy::Feedback,
                )?;
                if outcome.accepted {
                    return Ok(());
                }
                return Err(format!(
                    "generated workspace failed post-widen feedback verification; report written to {}",
                    report_path.display()
                )
                .into());
            }
            continue;
        }

        let signature = diagnostics_signature(&report.diagnostics);
        if !seen_diagnostics.insert(signature) {
            write_event_log(
                options,
                "feedback",
                "no_progress",
                "stop feedback loop",
                "feedback repeated identical diagnostics, so another attempt would not improve the slice",
                event_fields(&[("attempt", serde_json::json!(attempt))]),
            )?;
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "no_progress",
                "feedback made no diagnostic progress",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "failed",
                "feedback made no diagnostic progress",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback made no diagnostic progress; report written to {}",
                report_path.display()
            )
            .into());
        }
        let shape_signature = diagnostics_shape_signature(&report.diagnostics);
        if !seen_diagnostic_shapes.insert(shape_signature) {
            write_event_log(
                options,
                "feedback",
                "low_progress",
                "stop feedback loop",
                "feedback repeated the same diagnostic shape, so another attempt is likely cycling",
                event_fields(&[("attempt", serde_json::json!(attempt))]),
            )?;
            record_feedback_attempt(
                validation,
                "feedback",
                attempt,
                "low_progress",
                "feedback repeated the same diagnostic shape",
                &report,
                report_path.clone(),
                false,
                semantic_warnings,
                repairable_warning_count(&report.diagnostics),
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback",
                "failed",
                "feedback repeated the same diagnostic shape",
                &report,
                report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback repeated the same diagnostic shape; report written to {}",
                report_path.display()
            )
            .into());
        }
        record_feedback_attempt(
            validation,
            "feedback",
            attempt,
            "retrying",
            "generated workspace did not pass feedback gates",
            &report,
            report_path.clone(),
            false,
            semantic_warnings,
            repairable_warning_count(&report.diagnostics),
            None,
            None,
        );
    }

    validation.gates.push(ValidationGateReport {
        name: "feedback".to_string(),
        status: "failed".to_string(),
        reason: "generated workspace failed compiler feedback loop".to_string(),
        report_path: Some(report_path.clone()),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
    write_event_log(
        options,
        "feedback",
        "failed",
        "exhaust feedback loop",
        "generated workspace did not pass compiler feedback within configured attempts",
        event_fields(&[("attempts", serde_json::json!(options.feedback_iterations))]),
    )?;
    Err(format!(
        "generated workspace failed compiler feedback loop; report written to {}",
        report_path.display()
    )
    .into())
}

fn run_feedback_repair_loop(
    options: &CliOptions,
    baseline: Option<&CheckReport>,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    let feedback_report_path = options
        .feedback_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-feedback.json"));
    let repair_report_path = options
        .repair_report
        .clone()
        .unwrap_or_else(|| options.output_root.join("slice-repair.json"));
    let mut seen_diagnostics = std::collections::BTreeSet::new();
    let mut seen_diagnostic_shapes = std::collections::BTreeSet::new();
    let mut widening_state = FeedbackWideningState::default();
    let mut repaired_previous_attempt = false;

    for attempt in 1..=options.feedback_repair_iterations {
        println!(
            "feedback repair attempt {attempt}/{}: cargo check --message-format=json",
            options.feedback_repair_iterations
        );
        write_event_log(
            options,
            "feedback_repair_attempt",
            "started",
            "run cargo check before optional conservative repair",
            "compiler output decides whether the slice is accepted, repaired, widened, retried, or rejected",
            event_fields(&[
                ("attempt", serde_json::json!(attempt)),
                ("target_dir", serde_json::json!(feedback_target_dir(options))),
                (
                    "feedback_report",
                    serde_json::json!(feedback_report_path),
                ),
                ("repair_report", serde_json::json!(repair_report_path)),
            ]),
        )?;
        let report = check_workspace(CheckOptions {
            manifest_path: options.output_root.join("Cargo.toml"),
            target_dir: Some(feedback_target_dir(options)),
            timeout: options.feedback_timeout,
            cargo_args: options.cargo_check_args.clone(),
        })?;
        write_report(&report, &feedback_report_path)?;
        print_feedback(&report, options.feedback_limit, &feedback_report_path);

        let warnings = report.warning_count();
        let semantic_warnings = semantic_hazard_warning_count(&report.diagnostics, baseline);
        let repairable_warnings = repairable_warning_count(&report.diagnostics);
        write_event_log(
            options,
            "feedback_repair_attempt",
            if report.success { "checked" } else { "failed" },
            "classify compiler feedback before repair",
            "semantic warnings, repairable warnings, and diagnostic progress choose the next action",
            event_fields(&[
                ("attempt", serde_json::json!(attempt)),
                ("success", serde_json::json!(report.success)),
                ("errors", serde_json::json!(report.error_count())),
                ("warnings", serde_json::json!(warnings)),
                ("semantic_warnings", serde_json::json!(semantic_warnings)),
                ("repairable_warnings", serde_json::json!(repairable_warnings)),
                ("timed_out", serde_json::json!(report.timed_out)),
                ("duration_ms", serde_json::json!(report.duration_ms)),
            ]),
        )?;
        if feedback_repair_is_accepted(&report, baseline, options.deny_warnings) {
            write_event_log(
                options,
                "feedback_repair",
                "accepted",
                "accept generated workspace after compiler feedback",
                "cargo check passed all repair feedback gates",
                event_fields(&[("attempt", serde_json::json!(attempt))]),
            )?;
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "accepted",
                "generated workspace cargo check passed all feedback gates",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.success && semantic_warnings > 0 {
            println!("feedback: semantic warning gate rejected {semantic_warnings} new warning(s)");
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "rejected",
                "semantic warning gate rejected generated workspace",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "semantic warning gate rejected generated workspace",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            break;
        }
        if report.success && repairable_warnings > 0 {
            println!(
                "feedback: cargo check passed but {repairable_warnings} repairable warning(s) remain; attempting conservative repair"
            );
        }
        if report.success && options.deny_warnings && repairable_warnings == 0 && warnings > 0 {
            println!("feedback: warnings denied by --deny-warnings and no conservative repair is available");
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "rejected",
                "warnings were denied and no conservative repair was available",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "warnings were denied and no conservative repair was available",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            break;
        }
        if options.allow_baseline_failures
            && baseline_limited_feedback_repair_is_accepted(
                &report,
                baseline,
                options.deny_warnings,
            )
        {
            println!(
                "feedback: generated errors match the source baseline; treating as baseline-limited pass"
            );
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                feedback_report_path.clone(),
                true,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "baseline_limited",
                "generated errors match the source baseline and remaining gates passed",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Ok(());
        }
        if report.timed_out {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "timed_out",
                "feedback cargo check timed out",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "feedback cargo check timed out",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            break;
        }

        if try_widen_from_feedback(
            options,
            validation,
            &mut widening_state,
            "feedback-repair",
            attempt,
            &report,
            &feedback_report_path,
            semantic_warnings,
            repairable_warnings,
        )? {
            repaired_previous_attempt = false;
            if attempt == options.feedback_repair_iterations {
                let outcome = run_post_widen_feedback_verification(
                    options,
                    baseline,
                    validation,
                    "feedback-repair",
                    "feedback-repair",
                    attempt + 1,
                    &feedback_report_path,
                    options.cargo_check_args.clone(),
                    FeedbackAcceptancePolicy::Repair,
                )?;
                if outcome.accepted {
                    return Ok(());
                }
                return Err(format!(
                    "generated workspace failed post-widen feedback repair verification; feedback report written to {}",
                    feedback_report_path.display()
                )
                .into());
            }
            continue;
        }

        let signature = diagnostics_signature(&report.diagnostics);
        if !seen_diagnostics.insert(signature) {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "no_progress",
                "feedback repair made no diagnostic progress",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "feedback repair made no diagnostic progress",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback repair made no diagnostic progress; report written to {}",
                feedback_report_path.display()
            )
            .into());
        }
        let shape_signature = diagnostics_shape_signature(&report.diagnostics);
        let shape_repeated = !seen_diagnostic_shapes.insert(shape_signature);
        if repaired_previous_attempt && report.error_count() > 0 && shape_repeated {
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt,
                "low_progress",
                "feedback repair repeated the same diagnostic shape after changing files",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "feedback repair repeated the same diagnostic shape after changing files",
                &report,
                feedback_report_path.clone(),
                semantic_warnings,
            );
            return Err(format!(
                "feedback repair repeated the same diagnostic shape after changing files; report written to {}",
                feedback_report_path.display()
            )
            .into());
        }

        let repair_report = repair_workspace(RepairOptions {
            output_root: options.output_root.clone(),
            diagnostics: report.diagnostics.clone(),
        })?;
        write_repair_report(&repair_report, &repair_report_path)?;
        println!(
            "repair: removed_items={}, removed_imports={}, normalized_paths={}, applied_suggestions={}, added_dead_code_allows={}, deferred_dead_code_allows={}, skipped_diagnostics={}, changed_files={}, total_changes={}; report: {}",
            repair_report.removed_items,
            repair_report.removed_imports,
            repair_report.normalized_paths,
            repair_report.applied_suggestions,
            repair_report.added_dead_code_allows,
            repair_report.deferred_dead_code_allows,
            repair_report.skipped_diagnostics,
            repair_report.changed_files.len(),
            repair_report.total_changes(),
            repair_report_path.display()
        );

        let repair_total_changes = repair_report.total_changes();
        write_event_log(
            options,
            "feedback_repair_action",
            if repair_total_changes > 0 {
                "changed"
            } else {
                "unchanged"
            },
            "apply conservative generated-workspace repair from compiler diagnostics",
            if repair_total_changes > 0 {
                "repair changed only generated output and the next attempt must prove progress"
            } else {
                "no safe conservative repair matched the current diagnostics"
            },
            event_fields(&[
                ("attempt", serde_json::json!(attempt)),
                (
                    "removed_items",
                    serde_json::json!(repair_report.removed_items),
                ),
                (
                    "removed_imports",
                    serde_json::json!(repair_report.removed_imports),
                ),
                (
                    "normalized_paths",
                    serde_json::json!(repair_report.normalized_paths),
                ),
                (
                    "applied_suggestions",
                    serde_json::json!(repair_report.applied_suggestions),
                ),
                (
                    "added_dead_code_allows",
                    serde_json::json!(repair_report.added_dead_code_allows),
                ),
                (
                    "deferred_dead_code_allows",
                    serde_json::json!(repair_report.deferred_dead_code_allows),
                ),
                (
                    "skipped_diagnostics",
                    serde_json::json!(repair_report.skipped_diagnostics),
                ),
                (
                    "changed_files",
                    serde_json::json!(repair_report.changed_files),
                ),
                ("total_changes", serde_json::json!(repair_total_changes)),
                ("repair_report", serde_json::json!(repair_report_path)),
            ]),
        )?;
        record_feedback_attempt(
            validation,
            "feedback-repair",
            attempt,
            if repair_total_changes > 0 {
                "repaired"
            } else {
                "unrepaired"
            },
            if repair_total_changes > 0 {
                "conservative repair changed the generated workspace"
            } else {
                "no conservative repair was available"
            },
            &report,
            feedback_report_path.clone(),
            false,
            semantic_warnings,
            repairable_warnings,
            Some(repair_report_path.clone()),
            Some(repair_total_changes),
        );

        if repair_total_changes == 0 {
            break;
        }
        repaired_previous_attempt = true;

        let preflight = run_preflight(options)?;
        if !preflight.success {
            return Err("repair produced a structurally invalid generated workspace".into());
        }
        refresh_generated_lockfile_for_locked_validation(options, validation)?;
        if attempt == options.feedback_repair_iterations {
            println!("feedback repair final verification: cargo check --message-format=json");
            write_event_log(
                options,
                "feedback_repair_verification",
                "started",
                "run final cargo check after last conservative repair",
                "a last-attempt repair must be verified before the repair loop can accept or reject",
                event_fields(&[
                    ("attempt", serde_json::json!(attempt + 1)),
                    ("target_dir", serde_json::json!(feedback_target_dir(options))),
                    (
                        "feedback_report",
                        serde_json::json!(feedback_report_path),
                    ),
                    ("repair_report", serde_json::json!(repair_report_path)),
                    ("previous_repair_changes", serde_json::json!(repair_total_changes)),
                ]),
            )?;
            let repaired_check = check_workspace(CheckOptions {
                manifest_path: options.output_root.join("Cargo.toml"),
                target_dir: Some(feedback_target_dir(options)),
                timeout: options.feedback_timeout,
                cargo_args: options.cargo_check_args.clone(),
            })?;
            write_report(&repaired_check, &feedback_report_path)?;
            print_feedback(
                &repaired_check,
                options.feedback_limit,
                &feedback_report_path,
            );
            let repaired_semantic_warnings =
                semantic_hazard_warning_count(&repaired_check.diagnostics, baseline);
            let repaired_repairable_warnings =
                repairable_warning_count(&repaired_check.diagnostics);
            write_event_log(
                options,
                "feedback_repair_verification",
                if repaired_check.success {
                    "checked"
                } else {
                    "failed"
                },
                "classify final compiler feedback after last repair",
                "final repaired feedback decides whether the generated slice is accepted or rejected",
                event_fields(&[
                    ("attempt", serde_json::json!(attempt + 1)),
                    ("success", serde_json::json!(repaired_check.success)),
                    (
                        "errors",
                        serde_json::json!(repaired_check.error_count()),
                    ),
                    (
                        "warnings",
                        serde_json::json!(repaired_check.warning_count()),
                    ),
                    (
                        "semantic_warnings",
                        serde_json::json!(repaired_semantic_warnings),
                    ),
                    (
                        "repairable_warnings",
                        serde_json::json!(repaired_repairable_warnings),
                    ),
                    ("timed_out", serde_json::json!(repaired_check.timed_out)),
                    ("duration_ms", serde_json::json!(repaired_check.duration_ms)),
                ]),
            )?;
            if feedback_repair_is_accepted(&repaired_check, baseline, options.deny_warnings) {
                write_event_log(
                    options,
                    "feedback_repair",
                    "accepted",
                    "accept generated workspace after final repair verification",
                    "cargo check passed after the last conservative repair",
                    event_fields(&[("attempt", serde_json::json!(attempt + 1))]),
                )?;
                record_feedback_attempt(
                    validation,
                    "feedback-repair",
                    attempt + 1,
                    "accepted",
                    "generated workspace cargo check passed after conservative repair",
                    &repaired_check,
                    feedback_report_path.clone(),
                    false,
                    repaired_semantic_warnings,
                    repaired_repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "feedback-repair",
                    "accepted",
                    "generated workspace cargo check passed after conservative repair",
                    &repaired_check,
                    feedback_report_path.clone(),
                    repaired_semantic_warnings,
                );
                return Ok(());
            }
            if options.allow_baseline_failures
                && baseline_limited_feedback_repair_is_accepted(
                    &repaired_check,
                    baseline,
                    options.deny_warnings,
                )
            {
                record_feedback_attempt(
                    validation,
                    "feedback-repair",
                    attempt + 1,
                    "baseline_limited",
                    "generated errors match the source baseline after conservative repair",
                    &repaired_check,
                    feedback_report_path.clone(),
                    true,
                    repaired_semantic_warnings,
                    repaired_repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "feedback-repair",
                    "baseline_limited",
                    "generated errors match the source baseline after conservative repair",
                    &repaired_check,
                    feedback_report_path.clone(),
                    repaired_semantic_warnings,
                );
                return Ok(());
            }
            record_feedback_attempt(
                validation,
                "feedback-repair",
                attempt + 1,
                "rejected_after_repair",
                "final verification after conservative repair did not pass",
                &repaired_check,
                feedback_report_path.clone(),
                false,
                repaired_semantic_warnings,
                repaired_repairable_warnings,
                None,
                None,
            );
            record_feedback_gate(
                validation,
                "feedback-repair",
                "failed",
                "final verification after conservative repair did not pass",
                &repaired_check,
                feedback_report_path.clone(),
                repaired_semantic_warnings,
            );
        }
    }

    if !validation
        .gates
        .iter()
        .any(|gate| gate.name == "feedback-repair")
    {
        validation.gates.push(ValidationGateReport {
            name: "feedback-repair".to_string(),
            status: "failed".to_string(),
            reason: "generated workspace failed compiler repair loop".to_string(),
            report_path: Some(feedback_report_path.clone()),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
    }
    Err(format!(
        "generated workspace failed compiler repair loop; feedback report written to {}, repair report written to {}",
        feedback_report_path.display(),
        repair_report_path.display()
    )
    .into())
}

fn run_production_validation_matrix(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
    validation: &mut ValidationReport,
) -> Result<(), Box<dyn std::error::Error>> {
    if !options.production_preset {
        return Ok(());
    }

    let entries = production_validation_matrix_entries(options, production);
    if entries.is_empty() {
        validation.gates.push(ValidationGateReport {
            name: "production_matrix".to_string(),
            status: "not_required".to_string(),
            reason: "no uncovered concrete feature cfg matrix entries were discovered".to_string(),
            report_path: slice_report_path(options),
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        });
        return Ok(());
    }

    let mut baseline_limited = false;
    for (index, entry) in entries.iter().enumerate() {
        let matrix_index = index + 1;
        let baseline_report_path = production_matrix_baseline_report_path(options, matrix_index);
        let feedback_report_path = production_matrix_feedback_report_path(options, matrix_index);
        println!(
            "production matrix {matrix_index}/{} ({}): cargo check --message-format=json {}",
            entries.len(),
            entry.name,
            entry.cargo_args.join(" ")
        );

        let baseline = run_baseline_check_with_args(
            options,
            entry.cargo_args.clone(),
            production_matrix_baseline_target_dir(options, matrix_index),
        )?;
        write_report(&baseline, &baseline_report_path)?;
        validation.add_check_gate(
            "baseline-matrix",
            if baseline.success {
                "passed"
            } else if options.allow_baseline_failures {
                "baseline_allowed"
            } else {
                "failed"
            },
            if baseline.success {
                "source workspace matrix cargo check passed"
            } else if options.allow_baseline_failures {
                "source workspace matrix baseline failed but --allow-baseline-failures is enabled"
            } else {
                "source workspace matrix baseline failed"
            },
            &baseline,
            Some(baseline_report_path.clone()),
            0,
        );
        if !baseline.success && !options.allow_baseline_failures {
            return Err(format!(
                "source workspace failed production matrix baseline {}; report written to {}",
                entry.name,
                baseline_report_path.display()
            )
            .into());
        }

        let mut seen_diagnostics = BTreeSet::new();
        let mut seen_diagnostic_shapes = BTreeSet::new();
        let mut widening_state = FeedbackWideningState::default();
        let mut accepted = false;
        let mut entry_baseline_limited = false;
        for attempt in 1..=production_matrix_iterations(options) {
            let report = check_workspace(CheckOptions {
                manifest_path: options.output_root.join("Cargo.toml"),
                target_dir: Some(feedback_target_dir(options)),
                timeout: options.feedback_timeout,
                cargo_args: entry.cargo_args.clone(),
            })?;
            write_report(&report, &feedback_report_path)?;
            print_feedback(&report, options.feedback_limit, &feedback_report_path);

            let semantic_warnings =
                semantic_hazard_warning_count(&report.diagnostics, Some(&baseline));
            let repairable_warnings = repairable_warning_count(&report.diagnostics);
            if feedback_is_accepted(&report, Some(&baseline), options.deny_warnings) {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "accepted",
                    &entry.reason,
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                accepted = true;
                break;
            }
            if options.allow_baseline_failures
                && baseline_limited_feedback_is_accepted(
                    &report,
                    Some(&baseline),
                    options.deny_warnings,
                )
            {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "baseline_limited",
                    "generated matrix errors match the source matrix baseline",
                    &report,
                    feedback_report_path.clone(),
                    true,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                accepted = true;
                entry_baseline_limited = true;
                break;
            }
            if report.timed_out {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "timed_out",
                    "production matrix cargo check timed out",
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "production_matrix",
                    "failed",
                    "production matrix cargo check timed out",
                    &report,
                    feedback_report_path.clone(),
                    semantic_warnings,
                );
                return Err(format!(
                    "production matrix {} timed out; report written to {}",
                    entry.name,
                    feedback_report_path.display()
                )
                .into());
            }

            if try_widen_from_feedback(
                options,
                validation,
                &mut widening_state,
                "production-matrix",
                attempt,
                &report,
                &feedback_report_path,
                semantic_warnings,
                repairable_warnings,
            )? {
                if attempt == production_matrix_iterations(options) {
                    let outcome = run_post_widen_feedback_verification(
                        options,
                        Some(&baseline),
                        validation,
                        "production-matrix",
                        "production_matrix",
                        attempt + 1,
                        &feedback_report_path,
                        entry.cargo_args.clone(),
                        FeedbackAcceptancePolicy::Feedback,
                    )?;
                    if outcome.accepted {
                        accepted = true;
                        entry_baseline_limited = outcome.baseline_limited;
                        break;
                    }
                    return Err(format!(
                        "production matrix {} failed post-widen verification; report written to {}",
                        entry.name,
                        feedback_report_path.display()
                    )
                    .into());
                }
                continue;
            }

            let signature = diagnostics_signature(&report.diagnostics);
            if !seen_diagnostics.insert(signature) {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "no_progress",
                    "production matrix made no diagnostic progress",
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "production_matrix",
                    "failed",
                    "production matrix made no diagnostic progress",
                    &report,
                    feedback_report_path.clone(),
                    semantic_warnings,
                );
                return Err(format!(
                    "production matrix {} made no diagnostic progress; report written to {}",
                    entry.name,
                    feedback_report_path.display()
                )
                .into());
            }
            let shape_signature = diagnostics_shape_signature(&report.diagnostics);
            if !seen_diagnostic_shapes.insert(shape_signature) {
                record_feedback_attempt(
                    validation,
                    "production-matrix",
                    attempt,
                    "low_progress",
                    "production matrix repeated the same diagnostic shape",
                    &report,
                    feedback_report_path.clone(),
                    false,
                    semantic_warnings,
                    repairable_warnings,
                    None,
                    None,
                );
                record_feedback_gate(
                    validation,
                    "production_matrix",
                    "failed",
                    "production matrix repeated the same diagnostic shape",
                    &report,
                    feedback_report_path.clone(),
                    semantic_warnings,
                );
                return Err(format!(
                    "production matrix {} repeated the same diagnostic shape; report written to {}",
                    entry.name,
                    feedback_report_path.display()
                )
                .into());
            }
            record_feedback_attempt(
                validation,
                "production-matrix",
                attempt,
                "retrying",
                "generated workspace did not pass production matrix feedback",
                &report,
                feedback_report_path.clone(),
                false,
                semantic_warnings,
                repairable_warnings,
                None,
                None,
            );
        }

        if !accepted {
            validation.gates.push(ValidationGateReport {
                name: "production_matrix".to_string(),
                status: "failed".to_string(),
                reason: "generated workspace failed production matrix feedback".to_string(),
                report_path: Some(feedback_report_path.clone()),
                error_count: None,
                warning_count: None,
                semantic_warning_hazards: None,
                review_warning_hazards: None,
            });
            return Err(format!(
                "generated workspace failed production matrix {}; report written to {}",
                entry.name,
                feedback_report_path.display()
            )
            .into());
        }
        baseline_limited |= entry_baseline_limited;
    }

    validation.gates.push(ValidationGateReport {
        name: "production_matrix".to_string(),
        status: if baseline_limited {
            "baseline_limited"
        } else {
            "accepted"
        }
        .to_string(),
        reason: format!(
            "{} production feature matrix check(s) passed compiler feedback",
            entries.len()
        ),
        report_path: Some(production_matrix_feedback_report_path(
            options,
            entries.len(),
        )),
        error_count: Some(0),
        warning_count: Some(0),
        semantic_warning_hazards: Some(0),
        review_warning_hazards: Some(0),
    });
    Ok(())
}

fn production_validation_matrix_entries(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
) -> Vec<ProductionMatrixEntry> {
    if !options.production_preset
        || options
            .cargo_check_args
            .iter()
            .any(|arg| arg == "--all-features")
    {
        return Vec::new();
    }

    let mut uncovered_features = BTreeSet::new();
    for hazard in &production.hazards {
        if !matches!(
            hazard.code.as_str(),
            "cfg_gated_roots" | "conditional_compilation_attrs"
        ) {
            continue;
        }
        for detail in &hazard.details {
            if cfg_gate_detail_is_covered_by_args(options, detail) {
                continue;
            }
            let package = detail.package.as_deref();
            for feature in enabled_cargo_features(&detail.suggested_cargo_args) {
                if feature_is_enabled_for_matrix_detail(
                    &feature,
                    package,
                    &options.cargo_check_args,
                ) {
                    continue;
                }
                uncovered_features.insert(match package {
                    Some(package) if !feature.contains('/') => format!("{package}/{feature}"),
                    _ => feature,
                });
            }
        }
    }

    if uncovered_features.is_empty() {
        return Vec::new();
    }

    let mut cargo_args = options.cargo_check_args.clone();
    cargo_args.push("--features".to_string());
    cargo_args.push(
        uncovered_features
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(","),
    );
    vec![ProductionMatrixEntry {
        name: format!(
            "features:{}",
            uncovered_features
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(",")
        ),
        reason: "production matrix validated retained feature cfg surfaces".to_string(),
        cargo_args,
    }]
}

fn feature_is_enabled_for_matrix_detail(
    feature: &str,
    package: Option<&str>,
    cargo_args: &[String],
) -> bool {
    if cargo_args.iter().any(|arg| arg == "--all-features") {
        return true;
    }
    let enabled = enabled_cargo_features(cargo_args);
    enabled.contains(feature)
        || package
            .map(|package| enabled.contains(&format!("{package}/{feature}")))
            .unwrap_or(false)
}

fn production_matrix_iterations(options: &CliOptions) -> usize {
    options
        .feedback_repair_iterations
        .max(options.feedback_iterations)
        .max(1)
}

fn production_matrix_feedback_report_path(options: &CliOptions, index: usize) -> PathBuf {
    options
        .output_root
        .join(format!("slice-feedback-matrix-{index}.json"))
}

fn production_matrix_baseline_report_path(options: &CliOptions, index: usize) -> PathBuf {
    options
        .output_root
        .join(format!("slice-baseline-matrix-{index}.json"))
}

fn production_matrix_baseline_target_dir(options: &CliOptions, index: usize) -> PathBuf {
    sibling_output_path(
        &options.output_root,
        &format!("target-baseline-matrix-{index}"),
    )
}

fn feedback_target_dir(options: &CliOptions) -> PathBuf {
    options
        .feedback_target_dir
        .clone()
        .unwrap_or_else(|| options.output_root.join("target-feedback"))
}

fn production_readiness_gate_status(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
) -> String {
    if production.status == "hazards_detected"
        && options.production_preset
        && production_has_error_hazards(production)
        && !production_readiness_blocks_validation(options, production)
    {
        "requires_feedback".to_string()
    } else {
        production.status.clone()
    }
}

fn production_has_error_hazards(production: &opensource_core::ProductionReadinessReport) -> bool {
    production
        .hazards
        .iter()
        .any(|hazard| hazard.severity == "error")
}

fn review_warning_hazard_count(production: &opensource_core::ProductionReadinessReport) -> usize {
    production
        .hazards
        .iter()
        .filter(|hazard| {
            hazard.severity == "warning" && !production_warning_hazard_is_discharged(hazard)
        })
        .count()
}

fn production_warning_hazard_is_discharged(
    hazard: &opensource_core::ProductionHazardReport,
) -> bool {
    matches!(
        hazard.code.as_str(),
        "conditional_compilation_attrs"
            | "custom_attribute_macros"
            | "custom_derive_macros"
            | "custom_macro_invocations"
            | "dynamic_callback_boundaries"
            | "semantic_analyzer_unavailable"
            | "semantic_file_budget_exhausted"
            | "semantic_file_failures"
            | "semantic_inventory_available"
            | "semantic_inventory_not_applied"
            | "semantic_inventory_partially_applied"
            | "semantic_method_call_budget_exhausted"
            | "semantic_path_budget_exhausted"
            | "semantic_reduction_hints_applied"
            | "semantic_unresolved_method_calls"
            | "semantic_unresolved_paths"
            | "syntactic_method_fallback_cap"
            | "syntactic_method_fallbacks"
    )
}

fn production_readiness_blocks_validation(
    options: &CliOptions,
    production: &opensource_core::ProductionReadinessReport,
) -> bool {
    options.production_preset
        && production.status == "hazards_detected"
        && production.hazards.iter().any(|hazard| {
            hazard.severity == "error" && !production_error_hazard_is_discharged(options, hazard)
        })
}

fn production_error_hazard_is_discharged(
    options: &CliOptions,
    hazard: &opensource_core::ProductionHazardReport,
) -> bool {
    hazard.code == "cfg_gated_roots"
        && !hazard.details.is_empty()
        && hazard
            .details
            .iter()
            .all(|detail| cfg_gate_detail_is_covered_by_args(options, detail))
}

fn cfg_gate_detail_is_covered_by_args(
    options: &CliOptions,
    detail: &opensource_core::ProductionHazardDetail,
) -> bool {
    if let Some(expr) = detail.cfg.as_deref().and_then(cfg_attribute_expression) {
        let Some(cfg_set) = selected_rustc_cfg_set(&options.cargo_check_args) else {
            return false;
        };
        let enabled_features = enabled_cargo_features(&options.cargo_check_args);
        return evaluate_cfg_expr(
            &expr,
            &cfg_set,
            &enabled_features,
            detail.package.as_deref(),
            cargo_args_enable_all_features(&options.cargo_check_args),
        ) == CfgEval::True;
    }

    let required_features = enabled_cargo_features(&detail.suggested_cargo_args);
    !required_features.is_empty()
        && cargo_features_are_enabled(
            &required_features,
            detail.package.as_deref(),
            &enabled_cargo_features(&options.cargo_check_args),
            cargo_args_enable_all_features(&options.cargo_check_args),
        )
}

#[derive(Clone, Debug, Default)]
struct RustcCfgSet {
    values: BTreeMap<String, BTreeSet<String>>,
    flags: BTreeSet<String>,
}

impl RustcCfgSet {
    fn insert_value(&mut self, key: &str, value: &str) {
        self.values
            .entry(key.to_string())
            .or_default()
            .insert(value.to_string());
    }

    fn values(&self, key: &str) -> Option<&BTreeSet<String>> {
        self.values.get(key)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CfgExpr {
    All(Vec<CfgExpr>),
    Any(Vec<CfgExpr>),
    Not(Box<CfgExpr>),
    Flag(String),
    KeyValue { key: String, value: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CfgEval {
    True,
    False,
    Unknown,
}

impl CfgEval {
    fn and(values: impl IntoIterator<Item = CfgEval>) -> Self {
        let mut saw_unknown = false;
        for value in values {
            match value {
                CfgEval::True => {}
                CfgEval::False => return CfgEval::False,
                CfgEval::Unknown => saw_unknown = true,
            }
        }
        if saw_unknown {
            CfgEval::Unknown
        } else {
            CfgEval::True
        }
    }

    fn or(values: impl IntoIterator<Item = CfgEval>) -> Self {
        let mut saw_unknown = false;
        for value in values {
            match value {
                CfgEval::True => return CfgEval::True,
                CfgEval::False => {}
                CfgEval::Unknown => saw_unknown = true,
            }
        }
        if saw_unknown {
            CfgEval::Unknown
        } else {
            CfgEval::False
        }
    }

    fn not(self) -> Self {
        match self {
            CfgEval::True => CfgEval::False,
            CfgEval::False => CfgEval::True,
            CfgEval::Unknown => CfgEval::Unknown,
        }
    }
}

fn cfg_attribute_expression(cfg: &str) -> Option<CfgExpr> {
    let normalized = compact_cfg_attribute(cfg);
    if let Some(arguments) = cfg_call_arguments(&normalized, "cfg_attr") {
        return cfg_first_top_level_argument(arguments).and_then(parse_cfg_expr);
    }
    if let Some(arguments) = cfg_call_arguments(&normalized, "cfg") {
        return parse_cfg_expr(arguments);
    }
    None
}

fn compact_cfg_attribute(cfg: &str) -> String {
    let mut normalized = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in cfg.chars() {
        if in_string {
            normalized.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            normalized.push(ch);
        } else if !ch.is_whitespace() {
            normalized.push(ch);
        }
    }
    normalized
}

fn cfg_call_arguments<'a>(cfg: &'a str, call: &str) -> Option<&'a str> {
    let pattern = format!("{call}(");
    let start = cfg.find(&pattern)? + pattern.len();
    let end = matching_paren_index(cfg, start.checked_sub(1)?)?;
    Some(&cfg[start..end])
}

fn matching_paren_index(text: &str, open_index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = open_index;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn cfg_first_top_level_argument(arguments: &str) -> Option<&str> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in arguments.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.checked_sub(1)?;
        } else if byte == b',' && depth == 0 {
            return Some(&arguments[..index]);
        }
    }
    (!arguments.is_empty()).then_some(arguments)
}

fn parse_cfg_expr(text: &str) -> Option<CfgExpr> {
    let mut parser = CfgParser::new(text);
    let expr = parser.parse_expr()?;
    parser.is_finished().then_some(expr)
}

struct CfgParser<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> CfgParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, index: 0 }
    }

    fn is_finished(&self) -> bool {
        self.index == self.text.len()
    }

    fn parse_expr(&mut self) -> Option<CfgExpr> {
        let ident = self.parse_ident()?;
        if self.consume(b'(') {
            let args = self.parse_expr_list()?;
            return match ident.as_str() {
                "all" => Some(CfgExpr::All(args)),
                "any" => Some(CfgExpr::Any(args)),
                "not" if args.len() == 1 => Some(CfgExpr::Not(Box::new(args.into_iter().next()?))),
                _ => None,
            };
        }
        if self.consume(b'=') {
            let value = self.parse_string()?;
            return Some(CfgExpr::KeyValue { key: ident, value });
        }
        Some(CfgExpr::Flag(ident))
    }

    fn parse_expr_list(&mut self) -> Option<Vec<CfgExpr>> {
        if self.consume(b')') {
            return Some(Vec::new());
        }
        let mut args = Vec::new();
        loop {
            args.push(self.parse_expr()?);
            if self.consume(b')') {
                return Some(args);
            }
            if !self.consume(b',') {
                return None;
            }
        }
    }

    fn parse_ident(&mut self) -> Option<String> {
        let start = self.index;
        while let Some(byte) = self.peek() {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.index += 1;
            } else {
                break;
            }
        }
        (self.index > start).then(|| self.text[start..self.index].to_string())
    }

    fn parse_string(&mut self) -> Option<String> {
        if !self.consume(b'"') {
            return None;
        }
        let mut value = String::new();
        let mut escaped = false;
        while let Some(byte) = self.peek() {
            self.index += 1;
            let ch = byte as char;
            if escaped {
                value.push(ch);
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                return Some(value);
            } else {
                value.push(ch);
            }
        }
        None
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.index).copied()
    }
}

fn evaluate_cfg_expr(
    expr: &CfgExpr,
    cfg_set: &RustcCfgSet,
    enabled_features: &BTreeSet<String>,
    package: Option<&str>,
    all_features: bool,
) -> CfgEval {
    match expr {
        CfgExpr::All(args) => {
            CfgEval::and(args.iter().map(|arg| {
                evaluate_cfg_expr(arg, cfg_set, enabled_features, package, all_features)
            }))
        }
        CfgExpr::Any(args) => {
            CfgEval::or(args.iter().map(|arg| {
                evaluate_cfg_expr(arg, cfg_set, enabled_features, package, all_features)
            }))
        }
        CfgExpr::Not(arg) => {
            evaluate_cfg_expr(arg, cfg_set, enabled_features, package, all_features).not()
        }
        CfgExpr::Flag(flag) => evaluate_cfg_flag(flag, cfg_set),
        CfgExpr::KeyValue { key, value } => {
            evaluate_cfg_key_value(key, value, cfg_set, enabled_features, package, all_features)
        }
    }
}

fn evaluate_cfg_flag(flag: &str, cfg_set: &RustcCfgSet) -> CfgEval {
    if cfg_set.flags.contains(flag) {
        CfgEval::True
    } else if known_rustc_boolean_cfg(flag) {
        CfgEval::False
    } else {
        CfgEval::Unknown
    }
}

fn known_rustc_boolean_cfg(flag: &str) -> bool {
    matches!(
        flag,
        "debug_assertions" | "proc_macro" | "target_thread_local" | "unix" | "windows"
    )
}

fn evaluate_cfg_key_value(
    key: &str,
    value: &str,
    cfg_set: &RustcCfgSet,
    enabled_features: &BTreeSet<String>,
    package: Option<&str>,
    all_features: bool,
) -> CfgEval {
    if key == "feature" {
        return if cargo_feature_is_enabled(value, package, enabled_features, all_features) {
            CfgEval::True
        } else {
            CfgEval::Unknown
        };
    }
    if cfg_set
        .values(key)
        .is_some_and(|values| values.contains(value))
    {
        return CfgEval::True;
    }
    if known_rustc_value_cfg(key) {
        CfgEval::False
    } else {
        CfgEval::Unknown
    }
}

fn known_rustc_value_cfg(key: &str) -> bool {
    key == "panic" || key.starts_with("target_")
}

fn cargo_features_are_enabled(
    required_features: &BTreeSet<String>,
    package: Option<&str>,
    enabled_features: &BTreeSet<String>,
    all_features: bool,
) -> bool {
    all_features
        || required_features.iter().all(|feature| {
            cargo_feature_is_enabled(feature, package, enabled_features, all_features)
        })
}

fn cargo_feature_is_enabled(
    feature: &str,
    package: Option<&str>,
    enabled_features: &BTreeSet<String>,
    all_features: bool,
) -> bool {
    all_features
        || enabled_features.contains(feature)
        || package
            .map(|package| enabled_features.contains(&format!("{package}/{feature}")))
            .unwrap_or(false)
}

fn cargo_args_enable_all_features(cargo_args: &[String]) -> bool {
    cargo_args.iter().any(|arg| arg == "--all-features")
}

fn cargo_check_target(cargo_args: &[String]) -> Option<String> {
    let mut args = cargo_args.iter();
    while let Some(arg) = args.next() {
        if arg == "--target" {
            return args.next().cloned();
        }
        if let Some(target) = arg.strip_prefix("--target=") {
            return Some(target.to_string());
        }
    }
    None
}

fn selected_rustc_cfg_set(cargo_args: &[String]) -> Option<RustcCfgSet> {
    if let Some(target) = cargo_check_target(cargo_args) {
        return rustc_print_cfg(Some(&target)).or_else(|| target_triple_cfg_set(&target));
    }
    host_rustc_cfg_set()
}

fn host_rustc_cfg_set() -> Option<RustcCfgSet> {
    static HOST_CFG: OnceLock<Option<RustcCfgSet>> = OnceLock::new();
    HOST_CFG
        .get_or_init(|| rustc_print_cfg(None).or_else(host_const_cfg_set))
        .clone()
}

fn rustc_print_cfg(target: Option<&str>) -> Option<RustcCfgSet> {
    let mut command = Command::new("rustc");
    command.arg("--print").arg("cfg");
    if let Some(target) = target {
        command.arg("--target").arg(target);
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_rustc_cfg(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_rustc_cfg(text: &str) -> RustcCfgSet {
    let mut set = RustcCfgSet::default();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some((key, value)) = line.split_once("=\"") {
            if let Some(value) = value.strip_suffix('"') {
                set.insert_value(key, value);
            }
        } else {
            set.flags.insert(line.to_string());
        }
    }
    set
}

fn host_const_cfg_set() -> Option<RustcCfgSet> {
    let mut set = RustcCfgSet::default();
    set.insert_value("target_arch", std::env::consts::ARCH);
    set.insert_value("target_os", std::env::consts::OS);
    set.insert_value("target_family", std::env::consts::FAMILY);
    set.insert_value("target_pointer_width", &usize::BITS.to_string());
    set.insert_value(
        "target_endian",
        if cfg!(target_endian = "big") {
            "big"
        } else {
            "little"
        },
    );
    if cfg!(unix) {
        set.flags.insert("unix".to_string());
    }
    if cfg!(windows) {
        set.flags.insert("windows".to_string());
    }
    Some(set)
}

fn target_triple_cfg_set(target: &str) -> Option<RustcCfgSet> {
    let mut set = RustcCfgSet::default();
    set.insert_value("target_arch", target_arch(target)?);
    if let Some(vendor) = target_vendor(target) {
        set.insert_value("target_vendor", vendor);
    }
    if let Some(env) = target_env(target) {
        set.insert_value("target_env", env);
    }
    if let Some(os) = target_os(target) {
        set.insert_value("target_os", os);
    }
    if let Some(family) = target_family(target) {
        set.insert_value("target_family", family);
        set.flags.insert(family.to_string());
    }
    if let Some(pointer_width) = target_pointer_width(target) {
        set.insert_value("target_pointer_width", pointer_width);
    }
    if let Some(endian) = target_endian(target) {
        set.insert_value("target_endian", endian);
    }
    Some(set)
}

fn target_arch(target: &str) -> Option<&str> {
    target.split('-').next()
}

fn target_vendor(target: &str) -> Option<&str> {
    target.split('-').nth(1)
}

fn target_env(target: &str) -> Option<&str> {
    ["msvc", "gnu", "musl", "sgx", "newlib", "uclibc", "wasi"]
        .into_iter()
        .find(|env| target.split('-').any(|segment| segment == *env))
}

fn target_os(target: &str) -> Option<&'static str> {
    let segments = target.split('-').collect::<Vec<_>>();
    if segments.contains(&"linux") {
        Some("linux")
    } else if segments.contains(&"darwin") {
        Some("macos")
    } else if segments.contains(&"windows") {
        Some("windows")
    } else if segments.contains(&"android") {
        Some("android")
    } else if segments.contains(&"ios") {
        Some("ios")
    } else if segments.contains(&"freebsd") {
        Some("freebsd")
    } else if segments.contains(&"netbsd") {
        Some("netbsd")
    } else if segments.contains(&"openbsd") {
        Some("openbsd")
    } else if segments.contains(&"unknown") {
        Some("unknown")
    } else {
        None
    }
}

fn target_family(target: &str) -> Option<&'static str> {
    match target_os(target) {
        Some("windows") => Some("windows"),
        Some("linux" | "macos" | "android" | "ios" | "freebsd" | "netbsd" | "openbsd") => {
            Some("unix")
        }
        _ => None,
    }
}

fn target_pointer_width(target: &str) -> Option<&'static str> {
    let arch = target_arch(target)?;
    if arch.contains("64") || matches!(arch, "s390x") {
        Some("64")
    } else if arch.contains("32")
        || matches!(
            arch,
            "x86" | "i386" | "i586" | "i686" | "arm" | "thumb" | "mips"
        )
    {
        Some("32")
    } else {
        None
    }
}

fn target_endian(target: &str) -> Option<&'static str> {
    let arch = target_arch(target)?;
    if matches!(
        arch,
        "s390x" | "powerpc" | "powerpc64" | "mips" | "mips64" | "sparc"
    ) {
        Some("big")
    } else {
        Some("little")
    }
}

fn record_final_production_readiness(options: &CliOptions, validation: &mut ValidationReport) {
    if !options.production_preset {
        return;
    }
    if validation
        .gates
        .iter()
        .any(|gate| gate.name == "production_ready")
    {
        return;
    }

    let feedback_status = validation
        .gates
        .iter()
        .rev()
        .find(|gate| matches!(gate.name.as_str(), "feedback-repair" | "feedback"))
        .map(|gate| gate.status.as_str())
        .unwrap_or("missing_feedback");
    let production_readiness_status = validation
        .gates
        .iter()
        .rev()
        .find(|gate| gate.name == "production_readiness")
        .map(|gate| gate.status.as_str())
        .unwrap_or("missing_production_readiness");
    let review_warning_hazards = validation
        .gates
        .iter()
        .rev()
        .find(|gate| gate.name == "production_readiness")
        .and_then(|gate| gate.review_warning_hazards)
        .unwrap_or(usize::MAX);
    let production_matrix_status = validation
        .gates
        .iter()
        .rev()
        .find(|gate| gate.name == "production_matrix")
        .map(|gate| gate.status.as_str())
        .unwrap_or("not_required");
    let (status, reason) = match feedback_status {
        _ if production_readiness_status == "hazards_detected" => (
            "failed",
            "production readiness reported error hazards for the final generated slice",
        ),
        _ if production_matrix_status == "failed" => (
            "failed",
            "production matrix validation failed for the final generated slice",
        ),
        _ if production_matrix_status == "baseline_limited" || feedback_status == "baseline_limited" => (
            "baseline_limited",
            "production preset matched an allowed failing source baseline; generated workspace is not cleanly production-ready",
        ),
        "accepted"
            if production_readiness_status == "ready_for_feedback"
                && matches!(production_matrix_status, "accepted" | "not_required") =>
        {
            (
            "accepted",
            "production preset passed baseline, generation, preflight, target coverage, compiler feedback, and production hazard checks",
            )
        }
        "accepted"
            if review_warning_hazards == 0
                && matches!(production_matrix_status, "accepted" | "not_required") =>
        {
            (
                "accepted",
                "production preset passed compiler feedback and all remaining production warnings were discharged by feedback or matrix validation",
            )
        }
        "accepted" if matches!(production_matrix_status, "accepted" | "not_required") => (
            "review_required",
            "production preset passed compiler feedback, but remaining warning hazards require semantic review before production-ready acceptance",
        ),
        _ => (
            "failed",
            "production preset did not produce an accepted compiler feedback gate",
        ),
    };
    validation.gates.push(ValidationGateReport {
        name: "production_ready".to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        report_path: validation_report_path(options),
        error_count: None,
        warning_count: None,
        semantic_warning_hazards: None,
        review_warning_hazards: None,
    });
}

fn baseline_target_dir(options: &CliOptions) -> PathBuf {
    options.baseline_target_dir.clone().unwrap_or_else(|| {
        shared_baseline_target_dir(&options.workspace_root, &options.cargo_check_args)
    })
}

fn shared_baseline_target_dir(workspace_root: &Path, cargo_args: &[String]) -> PathBuf {
    let mut key_parts = Vec::with_capacity(cargo_args.len() + 2);
    key_parts.push("v1".to_string());
    key_parts.push(cache_key_path_component(workspace_root));
    key_parts.extend(cargo_args.iter().cloned());
    let key = stable_hex_hash(&key_parts);
    std::env::temp_dir()
        .join("slicers-baseline-targets")
        .join(key)
}

fn cache_key_path_component(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };
    fs::canonicalize(&absolute)
        .unwrap_or(absolute)
        .to_string_lossy()
        .into_owned()
}

fn stable_hex_hash(parts: &[String]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes().iter().chain(std::iter::once(&0)) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

fn sibling_output_path(output_root: &Path, suffix: &str) -> PathBuf {
    let Some(name) = output_root.file_name() else {
        return output_root.join(format!(".{suffix}"));
    };
    output_root.with_file_name(format!("{}-{suffix}", name.to_string_lossy()))
}

fn feedback_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    report.success
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
        && (!deny_warnings || report.warning_count() == 0)
}

fn feedback_repair_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    feedback_is_accepted(report, baseline, deny_warnings)
        && repairable_warning_count(&report.diagnostics) == 0
}

fn repair_has_deferred_warning_allows(report: &RepairReport) -> bool {
    report.deferred_dead_code_allows > 0 || report.deferred_lint_allows > 0
}

fn should_run_deferred_warning_repair(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
    saw_deferred_warning_allows: bool,
) -> bool {
    deny_warnings
        && saw_deferred_warning_allows
        && report.error_count() == 0
        && report.warning_count() > 0
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
}

fn baseline_limited_feedback_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    feedback_errors_are_baseline_known(report, baseline)
        && semantic_hazard_warning_count(&report.diagnostics, baseline) == 0
        && (!deny_warnings || report.warning_count() == 0)
}

fn baseline_limited_feedback_repair_is_accepted(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
    deny_warnings: bool,
) -> bool {
    baseline_limited_feedback_is_accepted(report, baseline, deny_warnings)
        && repairable_warning_count(&report.diagnostics) == 0
}

fn feedback_errors_are_baseline_known(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
) -> bool {
    let Some(baseline) = baseline else {
        return false;
    };
    if baseline.success || report.success {
        return false;
    }

    let baseline_errors = diagnostic_error_keys(&baseline.diagnostics);
    if baseline_errors.is_empty() {
        return false;
    }
    let generated_errors = diagnostic_error_keys(&report.diagnostics);
    !generated_errors.is_empty()
        && generated_errors
            .iter()
            .all(|error| baseline_errors.contains(error))
}

fn feedback_baseline_error_comparison(
    report: &CheckReport,
    baseline: Option<&CheckReport>,
) -> BaselineErrorComparison {
    let Some(baseline) = baseline else {
        return BaselineErrorComparison::default();
    };
    let baseline_errors = diagnostic_error_keys(&baseline.diagnostics);
    let mut comparison = BaselineErrorComparison {
        compared: true,
        known_errors: 0,
        new_errors: 0,
    };
    for diagnostic in report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.level == "error")
    {
        if baseline_errors.contains(&diagnostic_baseline_key(diagnostic)) {
            comparison.known_errors += 1;
        } else {
            comparison.new_errors += 1;
        }
    }
    comparison
}

fn diagnostic_error_keys(diagnostics: &[CheckDiagnostic]) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.level == "error")
        .map(diagnostic_baseline_key)
        .collect()
}

fn diagnostic_baseline_key(diagnostic: &CheckDiagnostic) -> String {
    let target = diagnostic
        .target
        .as_ref()
        .map(|target| format!("{}:{}", target.kind.join(","), target.name))
        .unwrap_or_default();
    let primary_files = diagnostic_primary_files(diagnostic);
    let rendered_fingerprint = if primary_files.is_empty() {
        diagnostic
            .rendered
            .as_deref()
            .map(stable_text_fingerprint)
            .unwrap_or_default()
    } else {
        String::new()
    };
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        diagnostic.level,
        diagnostic.code.as_deref().unwrap_or(""),
        diagnostic.package_id.as_deref().unwrap_or(""),
        target,
        primary_files,
        rendered_fingerprint,
        diagnostic.message
    )
}

fn diagnostic_primary_files(diagnostic: &CheckDiagnostic) -> String {
    let mut files = diagnostic
        .spans
        .iter()
        .filter(|span| span.is_primary)
        .map(|span| span.file_name.as_str())
        .collect::<Vec<_>>();
    files.sort_unstable();
    files.dedup();
    files.join(",")
}

fn stable_text_fingerprint(text: &str) -> String {
    let mut hash = 14_695_981_039_346_656_037u64;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("{hash:016x}")
}

fn semantic_hazard_warning_count(
    diagnostics: &[CheckDiagnostic],
    baseline: Option<&CheckReport>,
) -> usize {
    let baseline_warnings = baseline
        .map(|report| diagnostic_semantic_warning_keys(&report.diagnostics))
        .unwrap_or_default();
    diagnostics
        .iter()
        .filter(|diagnostic| semantic_warning_is_hazard(diagnostic))
        .filter(|diagnostic| !baseline_warnings.contains(&diagnostic_baseline_key(diagnostic)))
        .count()
}

fn diagnostic_semantic_warning_keys(diagnostics: &[CheckDiagnostic]) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| semantic_warning_is_hazard(diagnostic))
        .map(diagnostic_baseline_key)
        .collect()
}

fn semantic_warning_is_hazard(diagnostic: &CheckDiagnostic) -> bool {
    if diagnostic.level != "warning" {
        return false;
    }
    matches!(
        diagnostic.code.as_deref(),
        Some("unreachable_patterns" | "irrefutable_let_patterns" | "bindings_with_variant_name")
    ) || (diagnostic.code.as_deref() == Some("non_snake_case")
        && diagnostic.message.contains("variable `"))
        || diagnostic.message.contains("unreachable pattern")
        || diagnostic.message.contains("irrefutable")
}

fn repairable_warning_count(diagnostics: &[CheckDiagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.level == "warning"
                && (diagnostic.code.as_deref() == Some("unused_imports")
                    || diagnostic.code.as_deref() == Some("unused_macros")
                    || (diagnostic.code.as_deref() == Some("dead_code")
                        && dead_code_warning_is_repairable(&diagnostic.message)))
        })
        .count()
}

fn dead_code_warning_is_repairable(message: &str) -> bool {
    message.contains("function `")
        || message.contains("method `")
        || message.contains("associated function `")
        || message.contains("associated items `")
        || message.contains("associated constant `")
        || message.contains("constant `")
        || message.contains("enum `")
        || message.contains("module `")
        || message.contains("static `")
        || message.contains("struct `")
        || message.contains("type alias `")
        || message.contains("union `")
        || message.contains("trait `")
        || message.contains("field `")
        || message.contains("fields `")
        || message.contains("variant `")
        || message.contains("variants `")
}

fn diagnostics_signature(diagnostics: &[CheckDiagnostic]) -> String {
    let mut parts = diagnostics
        .iter()
        .map(|diagnostic| {
            let spans = diagnostic
                .spans
                .iter()
                .filter(|span| span.is_primary)
                .map(|span| {
                    format!(
                        "{}:{}:{}:{}",
                        span.file_name, span.line_start, span.column_start, span.column_end
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}",
                diagnostic.level,
                diagnostic.code.as_deref().unwrap_or(""),
                diagnostic.message,
                spans
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts.join("\n")
}

fn diagnostics_shape_signature(diagnostics: &[CheckDiagnostic]) -> String {
    let mut parts = diagnostics
        .iter()
        .map(|diagnostic| {
            let mut files = diagnostic
                .spans
                .iter()
                .filter(|span| span.is_primary)
                .map(|span| span.file_name.as_str())
                .collect::<Vec<_>>();
            files.sort_unstable();
            files.dedup();
            let target = diagnostic
                .target
                .as_ref()
                .map(|target| format!("{}:{}", target.kind.join("+"), target.name))
                .unwrap_or_default();
            format!(
                "{}|{}|{}|{}|{}|{}",
                diagnostic.level,
                diagnostic.code.as_deref().unwrap_or(""),
                diagnostic.package_id.as_deref().unwrap_or(""),
                target,
                diagnostic.message,
                files.join(",")
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts.join("\n")
}

fn print_preflight(report: &PreflightReport, limit: usize, report_path: &Path) {
    if report.success {
        println!(
            "preflight: passed; packages={}, rust_files={}, local_path_deps={}, external_deps={}, build_scripts={}; report: {}",
            report.packages,
            report.rust_files,
            report.local_path_dependencies,
            report.external_dependencies,
            report.build_scripts,
            report_path.display()
        );
        return;
    }

    println!(
        "preflight: failed with {} error(s), {} warning(s); report: {}",
        report.error_count(),
        report.warning_count(),
        report_path.display()
    );

    for diagnostic in prioritized_preflight_diagnostics(&report.diagnostics)
        .into_iter()
        .take(limit)
    {
        print_preflight_diagnostic(diagnostic);
    }
}

fn prioritized_preflight_diagnostics(
    diagnostics: &[PreflightDiagnostic],
) -> Vec<&PreflightDiagnostic> {
    let mut prioritized = diagnostics.iter().collect::<Vec<_>>();
    prioritized.sort_by_key(|diagnostic| match diagnostic.level.as_str() {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    });
    prioritized
}

fn print_preflight_diagnostic(diagnostic: &PreflightDiagnostic) {
    println!(
        "  {}[{}]: {}",
        diagnostic.level, diagnostic.code, diagnostic.message
    );
    if let Some(path) = &diagnostic.path {
        println!("    at {}", path.display());
    }
}

fn print_baseline(report: &CheckReport, limit: usize, report_path: Option<&Path>) {
    let report_text = report_path
        .map(|path| format!("; report: {}", path.display()))
        .unwrap_or_default();
    if report.success {
        println!(
            "baseline: source cargo check passed with {} warning(s) in {}{}",
            report.warning_count(),
            format_duration_ms(report.duration_ms),
            report_text
        );
        return;
    }

    println!(
        "baseline: source cargo check failed with {} error(s), {} warning(s) in {}{}",
        report.error_count(),
        report.warning_count(),
        format_duration_ms(report.duration_ms),
        report_text
    );

    for diagnostic in prioritized_diagnostics(&report.diagnostics)
        .into_iter()
        .take(limit)
    {
        print_diagnostic(diagnostic);
    }
}

fn print_feedback(report: &CheckReport, limit: usize, report_path: &Path) {
    if report.success {
        println!(
            "feedback: cargo check passed with {} warning(s) in {}; report: {}",
            report.warning_count(),
            format_duration_ms(report.duration_ms),
            report_path.display()
        );
        return;
    }

    println!(
        "feedback: cargo check failed with {} error(s), {} warning(s), {} widening candidate(s), {} hazard(s) in {}; report: {}",
        report.error_count(),
        report.warning_count(),
        report.widening.candidates.len(),
        report.widening.hazards.len(),
        format_duration_ms(report.duration_ms),
        report_path.display()
    );

    for candidate in report.widening.candidates.iter().take(limit.min(4)) {
        let location = candidate
            .file_name
            .as_deref()
            .zip(candidate.line_start)
            .map(|(file, line)| format!(" at {file}:{line}"))
            .unwrap_or_default();
        let symbol = candidate
            .symbol
            .as_deref()
            .map(|symbol| format!(" `{symbol}`"))
            .unwrap_or_default();
        let suggestions = if candidate.machine_applicable_suggestions > 0 {
            format!(
                " ({} machine-applicable suggestion(s))",
                candidate.machine_applicable_suggestions
            )
        } else if candidate.suggestions > 0 {
            format!(" ({} compiler suggestion(s))", candidate.suggestions)
        } else {
            String::new()
        };
        println!(
            "  widening {}{}{}{}: {}",
            candidate.kind, symbol, location, suggestions, candidate.action
        );
    }

    for diagnostic in prioritized_diagnostics(&report.diagnostics)
        .into_iter()
        .take(limit)
    {
        print_diagnostic(diagnostic);
    }
}

fn prioritized_diagnostics(diagnostics: &[CheckDiagnostic]) -> Vec<&CheckDiagnostic> {
    let mut prioritized = diagnostics.iter().collect::<Vec<_>>();
    prioritized.sort_by_key(|diagnostic| match diagnostic.level.as_str() {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    });
    prioritized
}

fn print_diagnostic(diagnostic: &CheckDiagnostic) {
    let code = diagnostic
        .code
        .as_deref()
        .map(|code| format!("[{code}]"))
        .unwrap_or_default();
    println!("  {}{}: {}", diagnostic.level, code, diagnostic.message);

    for span in diagnostic
        .spans
        .iter()
        .filter(|span| span.is_primary)
        .take(2)
    {
        println!(
            "    at {}:{}:{}",
            span.file_name, span.line_start, span.column_start
        );
        if let Some(line) = span.text.first() {
            println!("    | {line}");
        }
    }
}

fn format_duration_ms(duration_ms: u64) -> String {
    if duration_ms < 1_000 {
        format!("{duration_ms}ms")
    } else {
        format!("{:.2}s", duration_ms as f64 / 1_000.0)
    }
}

fn usage() -> String {
    concat!(
        "usage: slicers [--analyzer <syn|ra-hir|ra-feedback|ra-hir-proc-macros>] [--production] [--check] [--preflight] [--feedback] ",
        "[--feedback-loop <n>] [--feedback-repair-loop <n>] [--feedback-limit <n>] ",
        "[--feedback-timeout <seconds>] [--deny-warnings] [--feedback-report <path>] ",
        "[--feedback-target-dir <path>] [--cargo-check-arg <arg>] [--repair-report <path>] ",
        "[--baseline-check] [--allow-baseline-failures] [--baseline-report <path>] ",
        "[--baseline-target-dir <path>] [--slice-report <path>] [--decision-log <path>] [--event-log <path>] [--validation-report <path>] ",
        "[--preflight-report <path>] [--root <selector>|--root-selector <selector>] [--roots-file <path>] ",
        "[--random-roots <n>] [--random-root-package <package>] [--random-seed <n>] [--batch-roots] [--batch-report <path>] ",
        "[--workspace-root <workspace-root-or-Cargo.toml>] [--output <output-root>] <workspace-root-or-Cargo.toml> <output-root>\n",
        "default analyzer: ra-hir when the binary is built with the ra-hir feature, otherwise syn; ",
        "ra-feedback exposes the bounded RA outgoing-call closure explicitly; ",
        "--production defaults to ra-hir-proc-macros with RA feedback closure when available; ",
        "--root selects functions/items in memory without editing source, while #[opensourced] roots still work"
    )
    .to_string()
}

fn same_path(left: &Path, right: &Path) -> bool {
    let Ok(left) = left.canonicalize() else {
        return false;
    };
    let Ok(right) = right.canonicalize() else {
        return false;
    };
    left == right
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, fs, path::PathBuf};

    use opensource_core::{
        AnalyzerMode, CallableId, CheckDiagnostic, CheckReport, CheckTarget,
        FeedbackWideningReport, GeneratedTargetReport, RootId, SemanticReport,
    };

    use super::{
        apply_default_marked_package_scope, baseline_limited_feedback_is_accepted,
        baseline_target_dir, batch_root_attempts, cargo_args_have_package_scope,
        decision_log_feedback_diagnostics, decision_log_path, diagnostics_shape_signature,
        diagnostics_signature, event_log_path, feedback_baseline_error_comparison,
        feedback_errors_are_baseline_known, feedback_is_accepted, feedback_repair_is_accepted,
        initialize_event_log, parse_args_from, production_readiness_blocks_validation,
        production_validation_matrix_entries, record_feedback_attempt,
        record_final_production_readiness, record_production_readiness_gate,
        refresh_generated_lockfile_for_locked_validation, run_batch_roots,
        run_feedback_repair_loop, run_plain_check_gate, semantic_hazard_warning_count,
        semantic_proof_block_reason, semantic_proof_status, should_run_deferred_warning_repair,
        slice_report_path, try_widen_from_feedback, uncovered_validation_targets,
        validation_report_path, write_feedback_diagnostics_event, write_report,
        FeedbackWideningState, ValidationGateReport, ValidationReport,
    };

    #[test]
    fn semantic_proof_status_allows_selected_root_complete_with_workspace_budget_limits() {
        let semantic = SemanticReport {
            source_files: 120,
            analyzed_files: 48,
            skipped_files: 72,
            unqueried_method_calls: 1301,
            unqueried_paths: 10933,
            selected_root_source_files: 1,
            selected_root_analyzed_files: 1,
            ..SemanticReport::default()
        };

        assert_eq!(
            semantic_proof_status(&semantic),
            "selected_root_complete_workspace_limited"
        );
        assert!(semantic_proof_block_reason(Some(&semantic)).is_none());
    }

    #[test]
    fn semantic_proof_status_blocks_selected_root_budget_limits() {
        let semantic = SemanticReport {
            source_files: 7,
            analyzed_files: 7,
            selected_root_source_files: 1,
            selected_root_analyzed_files: 1,
            selected_root_unqueried_paths: 3,
            ..SemanticReport::default()
        };

        assert_eq!(semantic_proof_status(&semantic), "selected_root_limited");
        assert!(semantic_proof_block_reason(Some(&semantic))
            .expect("selected-root limits should block production")
            .contains("selected root semantic proof is budget-limited"));
    }

    #[test]
    fn semantic_proof_status_blocks_missing_ra_semantics() {
        assert_eq!(
            semantic_proof_block_reason(None).as_deref(),
            Some("rust-analyzer semantic proof is not available")
        );
    }

    #[test]
    fn accepts_warning_bearing_feedback_only_when_warning_denial_is_disabled() {
        let clean = report(true, Vec::new());
        let warning_report = report(false, vec![warning("unused variable: `value`")]);
        let mut successful_with_warning = warning_report.clone();
        successful_with_warning.success = true;

        assert!(feedback_is_accepted(&clean, None, true));
        assert!(feedback_is_accepted(&successful_with_warning, None, false));
        assert!(!feedback_is_accepted(&successful_with_warning, None, true));
    }

    #[test]
    fn repair_loop_does_not_accept_repairable_warnings_before_cleanup() {
        let report = report(
            true,
            vec![warning_with_code(
                "unused_imports",
                "unused import: `std::fmt`",
            )],
        );

        assert!(feedback_is_accepted(&report, None, false));
        assert!(!feedback_repair_is_accepted(&report, None, false));
    }

    #[test]
    fn feedback_repair_loop_verifies_last_attempt_warning_cleanup() {
        let output = temp_path("feedback-repair-final-verification-output");
        write(
            output.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            output.join("src/lib.rs"),
            r#"use std::{fmt, mem};

pub fn selected() -> usize {
    mem::size_of::<usize>()
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--feedback-repair-loop"),
            OsString::from("1"),
            OsString::from("--feedback-timeout"),
            OsString::from("60"),
            OsString::from("source"),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);

        run_feedback_repair_loop(&options, None, &mut validation)
            .expect("last-attempt warning repair should be verified and accepted");

        let repaired = fs::read_to_string(output.join("src/lib.rs")).unwrap();
        assert!(!repaired.contains("fmt"), "{repaired}");
        assert!(validation
            .attempts
            .iter()
            .any(|attempt| attempt.status == "repaired"));
        assert!(validation
            .attempts
            .iter()
            .any(|attempt| attempt.status == "accepted"));
        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "feedback-repair")
            .expect("feedback repair gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn deferred_warning_repair_runs_only_for_denied_warning_only_reports() {
        let warning_report = report(
            true,
            vec![warning_with_code(
                "private_interfaces",
                "type `Private` is more private than the item `Public::field`",
            )],
        );
        let error_report = report(
            false,
            vec![
                diagnostic("E0425", "cannot find value `missing` in this scope"),
                warning_with_code(
                    "private_interfaces",
                    "type `Private` is more private than the item `Public::field`",
                ),
            ],
        );
        let semantic_warning_report = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );

        assert!(should_run_deferred_warning_repair(
            &warning_report,
            None,
            true,
            true
        ));
        assert!(!should_run_deferred_warning_repair(
            &warning_report,
            None,
            false,
            true
        ));
        assert!(!should_run_deferred_warning_repair(
            &warning_report,
            None,
            true,
            false
        ));
        assert!(!should_run_deferred_warning_repair(
            &error_report,
            None,
            true,
            true
        ));
        assert!(!should_run_deferred_warning_repair(
            &semantic_warning_report,
            None,
            true,
            true
        ));
    }

    #[test]
    fn rejects_new_semantic_warning_even_when_warning_denial_is_disabled() {
        let report = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );

        assert_eq!(semantic_hazard_warning_count(&report.diagnostics, None), 1);
        assert!(!feedback_is_accepted(&report, None, false));
    }

    #[test]
    fn rejects_const_pattern_binding_warning_as_semantic_hazard() {
        let report = report(
            true,
            vec![warning_with_code(
                "non_snake_case",
                "variable `HTTP_OK` should have a snake case name",
            )],
        );

        assert_eq!(semantic_hazard_warning_count(&report.diagnostics, None), 1);
        assert!(!feedback_is_accepted(&report, None, false));
    }

    #[test]
    fn accepts_semantic_warning_already_present_in_source_baseline() {
        let baseline = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );
        let generated = report(
            true,
            vec![warning_with_code(
                "unreachable_patterns",
                "unreachable pattern",
            )],
        );

        assert_eq!(
            semantic_hazard_warning_count(&generated.diagnostics, Some(&baseline)),
            0
        );
        assert!(feedback_is_accepted(&generated, Some(&baseline), false));
    }

    #[test]
    fn recognizes_generated_errors_present_in_failed_source_baseline() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );

        assert!(feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_error_comparison_counts_known_and_new_generated_errors() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![
                diagnostic("E0425", "cannot find value `x` in this scope"),
                diagnostic("E0432", "unresolved import `crate::missing`"),
            ],
        );

        let comparison = feedback_baseline_error_comparison(&generated, Some(&baseline));
        assert!(comparison.compared);
        assert_eq!(comparison.known_errors, 1);
        assert_eq!(comparison.new_errors, 1);

        let no_baseline = feedback_baseline_error_comparison(&generated, None);
        assert!(!no_baseline.compared);
        assert_eq!(no_baseline.known_errors, 0);
        assert_eq!(no_baseline.new_errors, 0);
    }

    #[test]
    fn batch_row_records_baseline_error_attribution() {
        let options = parse_options(["--check", "workspace", "out"]);
        let root = RootId::Callable(CallableId::Free {
            package: "app".to_string(),
            module_path: Vec::new(),
            name: "selected".to_string(),
        });
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![
                diagnostic("E0425", "cannot find value `x` in this scope"),
                diagnostic("E0432", "unresolved import `crate::missing`"),
            ],
        );

        let row = super::batch_row_from_reports(
            &options,
            &root,
            &PathBuf::from("/tmp/slice"),
            "check_failed",
            None,
            None,
            Some(&generated),
            Some(&baseline),
            None,
        );

        assert_eq!(row.feedback_baseline_compared, Some(true));
        assert_eq!(row.feedback_baseline_known_errors, Some(1));
        assert_eq!(row.feedback_baseline_new_errors, Some(1));
    }

    #[test]
    fn baseline_limited_feedback_still_honors_warning_denial() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![
                diagnostic("E0425", "cannot find value `x` in this scope"),
                warning("unused variable: `value`"),
            ],
        );

        assert!(baseline_limited_feedback_is_accepted(
            &generated,
            Some(&baseline),
            false
        ));
        assert!(!baseline_limited_feedback_is_accepted(
            &generated,
            Some(&baseline),
            true
        ));
    }

    #[test]
    fn rejects_generated_errors_not_present_in_source_baseline() {
        let baseline = report(
            false,
            vec![diagnostic("E0425", "cannot find value `x` in this scope")],
        );
        let generated = report(
            false,
            vec![diagnostic("E0432", "unresolved import `crate::missing`")],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_matching_keeps_package_and_target_boundaries() {
        let baseline = report(
            false,
            vec![diagnostic_for_target(
                "E0425",
                "cannot find value `x` in this scope",
                "pkg_a 0.1.0",
                "lib",
                "pkg_a",
            )],
        );
        let generated = report(
            false,
            vec![diagnostic_for_target(
                "E0425",
                "cannot find value `x` in this scope",
                "pkg_b 0.1.0",
                "bin",
                "pkg_b",
            )],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_matching_keeps_primary_span_files() {
        let baseline = report(
            false,
            vec![diagnostic_with_span(
                "E0425",
                "cannot find value `x` in this scope",
                "src/lib.rs",
                12,
                9,
                10,
            )],
        );
        let generated = report(
            false,
            vec![diagnostic_with_span(
                "E0425",
                "cannot find value `x` in this scope",
                "src/main.rs",
                12,
                9,
                10,
            )],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn baseline_matching_distinguishes_stderr_only_failures() {
        let baseline = report(
            false,
            vec![diagnostic_with_rendered(
                "cargo-stderr",
                "error: failed to select a version",
                "error: failed to select a version\ncandidate A\n",
            )],
        );
        let generated = report(
            false,
            vec![diagnostic_with_rendered(
                "cargo-stderr",
                "error: failed to select a version",
                "error: failed to select a version\ncandidate B\n",
            )],
        );

        assert!(!feedback_errors_are_baseline_known(
            &generated,
            Some(&baseline)
        ));
    }

    #[test]
    fn diagnostic_signatures_are_order_insensitive() {
        let left = vec![
            diagnostic("E0432", "unresolved import `crate::missing`"),
            diagnostic("E0425", "cannot find value `x` in this scope"),
        ];
        let right = vec![
            diagnostic("E0425", "cannot find value `x` in this scope"),
            diagnostic("E0432", "unresolved import `crate::missing`"),
        ];

        assert_eq!(diagnostics_signature(&left), diagnostics_signature(&right));
    }

    #[test]
    fn diagnostic_shape_signatures_ignore_line_churn_but_keep_files() {
        let original = vec![diagnostic_with_span(
            "E0425",
            "cannot find value `x` in this scope",
            "src/lib.rs",
            12,
            9,
            10,
        )];
        let shifted = vec![diagnostic_with_span(
            "E0425",
            "cannot find value `x` in this scope",
            "src/lib.rs",
            20,
            5,
            6,
        )];
        let other_file = vec![diagnostic_with_span(
            "E0425",
            "cannot find value `x` in this scope",
            "src/main.rs",
            20,
            5,
            6,
        )];

        assert_ne!(
            diagnostics_signature(&original),
            diagnostics_signature(&shifted)
        );
        assert_eq!(
            diagnostics_shape_signature(&original),
            diagnostics_shape_signature(&shifted)
        );
        assert_ne!(
            diagnostics_shape_signature(&original),
            diagnostics_shape_signature(&other_file)
        );
    }

    #[test]
    fn production_flag_enables_strict_validation_preset() {
        let options = parse_options(["--production", "workspace", "out"]);

        assert!(options.run_baseline_check);
        assert!(options.run_preflight);
        assert!(options.deny_warnings);
        assert!(options.production_preset);
        assert_eq!(options.feedback_repair_iterations, 3);
        assert_eq!(options.feedback_iterations, 0);
        assert_eq!(
            options.analyzer_mode,
            AnalyzerMode::production_default_for_build()
        );
        assert_eq!(
            slice_report_path(&options),
            Some(PathBuf::from("out/slice-report.json"))
        );
        assert_eq!(
            validation_report_path(&options),
            Some(PathBuf::from("out/slice-validation.json"))
        );
        assert_eq!(
            decision_log_path(&options),
            Some(PathBuf::from("out/slice-decision-log.json"))
        );
        assert_eq!(
            event_log_path(&options),
            Some(PathBuf::from("out-slice-events.jsonl"))
        );
        assert_eq!(options.workspace_root, PathBuf::from("workspace"));
        assert_eq!(options.output_root, PathBuf::from("out"));
    }

    #[test]
    fn default_baseline_target_is_shared_for_same_source_workspace() {
        let first = parse_options(["--production", "workspace", "out-a"]);
        let second = parse_options(["--production", "workspace", "out-b"]);

        assert_eq!(baseline_target_dir(&first), baseline_target_dir(&second));
        assert!(baseline_target_dir(&first)
            .to_string_lossy()
            .contains("slicers-baseline-targets"));
    }

    #[test]
    fn baseline_target_cache_key_includes_cargo_check_args() {
        let default_args = parse_options(["--production", "workspace", "out"]);
        let feature_args = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "mobile",
            "workspace",
            "out",
        ]);

        assert_ne!(
            baseline_target_dir(&default_args),
            baseline_target_dir(&feature_args)
        );
    }

    #[test]
    fn explicit_baseline_target_dir_overrides_shared_default() {
        let options = parse_options([
            "--production",
            "--baseline-target-dir",
            "custom-target",
            "workspace",
            "out",
        ]);

        assert_eq!(
            baseline_target_dir(&options),
            PathBuf::from("custom-target")
        );
    }

    #[test]
    fn default_analyzer_matches_build_features() {
        let options = parse_options(["workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::default_for_build());
        #[cfg(feature = "ra-hir")]
        assert_eq!(options.analyzer_mode, AnalyzerMode::RustAnalyzerHir);
        #[cfg(not(feature = "ra-hir"))]
        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
    }

    #[test]
    fn explicit_syn_analyzer_overrides_semantic_default() {
        let options = parse_options(["--analyzer", "syn", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
    }

    #[test]
    fn explicit_ra_feedback_analyzer_is_accepted() {
        let options = parse_options(["--analyzer", "ra-feedback", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::RustAnalyzerFeedback);
    }

    #[test]
    fn explicit_syn_analyzer_overrides_production_semantic_default() {
        let options = parse_options(["--production", "--analyzer", "syn", "workspace", "out"]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
        assert!(options.production_preset);
    }

    #[test]
    fn default_validation_scope_targets_marked_packages() {
        let workspace = temp_path("cli-marked-package-scope-source");
        let output = temp_path("cli-marked-package-scope-output");
        write(
            workspace.join("Cargo.toml"),
            r#"[workspace]
members = ["app", "unrelated"]
resolver = "2"
"#,
        );
        write(
            workspace.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            workspace.join("app/src/lib.rs"),
            "#[opensourced::opensourced]\npub fn selected() -> i32 { 1 }\n",
        );
        write(
            workspace.join("unrelated/Cargo.toml"),
            "[package]\nname = \"unrelated\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            workspace.join("unrelated/src/lib.rs"),
            "compile_error!(\"unrelated package should not define validation scope\");\n",
        );
        let mut options = parse_args_from([
            std::ffi::OsString::from("--production"),
            workspace.clone().into_os_string(),
            output.into_os_string(),
        ])
        .expect("production arguments should parse");

        apply_default_marked_package_scope(&mut options)
            .expect("marked package scope should resolve");

        assert_eq!(options.cargo_check_args, ["-p", "app"]);
    }

    #[test]
    fn default_validation_scope_respects_explicit_package_selection() {
        let mut options = parse_options([
            "--production",
            "--cargo-check-arg",
            "--workspace",
            "workspace",
            "out",
        ]);
        let original_args = options.cargo_check_args.clone();

        apply_default_marked_package_scope(&mut options)
            .expect("explicit workspace scope should skip marker scan");

        assert_eq!(options.cargo_check_args, original_args);
    }

    #[test]
    fn default_validation_scope_skips_marker_scan_for_explicit_roots() {
        let mut options = parse_options([
            "--production",
            "--root",
            "app::selected",
            "/definitely/missing/workspace",
            "out",
        ]);
        let original_args = options.cargo_check_args.clone();

        apply_default_marked_package_scope(&mut options)
            .expect("explicit root scope should not inspect source markers");

        assert_eq!(options.cargo_check_args, original_args);
    }

    #[test]
    fn cargo_arg_package_scope_detection_accepts_common_forms() {
        assert!(cargo_args_have_package_scope(&["-p".to_string()]));
        assert!(cargo_args_have_package_scope(&["-papp".to_string()]));
        assert!(cargo_args_have_package_scope(
            &["--package=app".to_string()]
        ));
        assert!(cargo_args_have_package_scope(&["--workspace".to_string()]));
        assert!(!cargo_args_have_package_scope(&[
            "--all-targets".to_string()
        ]));
    }

    #[test]
    fn production_preset_locks_existing_source_lockfile() {
        let workspace = temp_path("cli-production-locked-source");
        let output = temp_path("cli-production-locked-output");
        write(workspace.join("Cargo.toml"), "[workspace]\nmembers = []\n");
        write(workspace.join("Cargo.lock"), "# lockfile\n");

        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            workspace.into_os_string(),
            output.into_os_string(),
        ])
        .expect("arguments should parse");

        assert!(options.cargo_check_args.iter().any(|arg| arg == "--locked"));
    }

    #[test]
    fn production_preset_does_not_duplicate_locking_args() {
        let workspace = temp_path("cli-production-frozen-source");
        let output = temp_path("cli-production-frozen-output");
        write(workspace.join("Cargo.toml"), "[workspace]\nmembers = []\n");
        write(workspace.join("Cargo.lock"), "# lockfile\n");

        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            std::ffi::OsString::from("--cargo-check-arg"),
            std::ffi::OsString::from("--frozen"),
            workspace.into_os_string(),
            output.into_os_string(),
        ])
        .expect("arguments should parse");

        assert_eq!(options.cargo_check_args, ["--frozen"]);
    }

    #[test]
    fn locked_validation_reconciles_generated_lockfile_before_check() {
        let source = temp_path("cli-lockfile-source");
        let output = temp_path("cli-lockfile-output");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\n",
        );
        write(source.join("Cargo.lock"), stale_generated_lockfile());
        write(
            output.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\n",
        );
        write(
            output.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(output.join("app/src/lib.rs"), "");
        write(output.join("Cargo.lock"), stale_generated_lockfile());

        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            source.into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);

        refresh_generated_lockfile_for_locked_validation(&options, &mut validation)
            .expect("generated lockfile should reconcile");

        let lockfile = fs::read_to_string(output.join("Cargo.lock")).unwrap();
        assert!(!lockfile.contains("\"dead-helper\""));
        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "lockfile")
            .expect("lockfile gate should be recorded");
        assert_eq!(gate.status, "passed");
    }

    #[test]
    fn production_preset_fails_closed_on_error_readiness_hazards() {
        let production = parse_options(["--production", "workspace", "out"]);
        let feedback_only = parse_options(["--feedback", "workspace", "out"]);
        let error_hazard = production_report(vec![production_hazard(
            "function_pointer_surfaces",
            "error",
        )]);
        let feedback_hazard = production_report(vec![production_hazard(
            "custom_macro_invocations",
            "warning",
        )]);

        assert!(production_readiness_blocks_validation(
            &production,
            &error_hazard
        ));
        assert!(!production_readiness_blocks_validation(
            &production,
            &feedback_hazard
        ));
        assert!(!production_readiness_blocks_validation(
            &feedback_only,
            &error_hazard
        ));
    }

    #[test]
    fn production_preset_discharges_feature_cfg_root_hazards_when_args_cover_them() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "selected",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![feature_cfg_root_hazard("app", "selected")]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));

        let mut validation = ValidationReport::new(&production);
        record_production_readiness_gate(
            &production,
            &mut validation,
            &report,
            "before compiler feedback",
        );
        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_readiness")
            .expect("production readiness gate should be recorded");
        assert_eq!(gate.status, "requires_feedback");
        assert_eq!(gate.error_count, Some(1));
        assert!(gate.reason.contains("feature cfg error hazard(s) covered"));
    }

    #[test]
    fn production_preset_discharges_feature_cfg_root_hazards_with_all_features() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--all-features",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![feature_cfg_root_hazard("app", "selected")]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_simple_target_cfg_root_hazards_with_matching_target() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--target",
            "--cargo-check-arg",
            "wasm32-unknown-unknown",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            r#"#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]"#,
        )]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_host_cfg_root_hazards_without_explicit_target() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(unix)]"
        } else {
            "#[cfg(windows)]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_any_target_cfg_root_hazards_when_one_branch_matches() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(any(windows, unix))]"
        } else {
            "#[cfg(any(unix, windows))]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_not_target_cfg_root_hazards_when_branch_is_false() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(not(windows))]"
        } else {
            "#[cfg(not(unix))]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_all_feature_and_target_cfg_root_hazards() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "selected",
            "workspace",
            "out",
        ]);
        let target_flag = if cfg!(unix) { "unix" } else { "windows" };
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            &format!(r#"#[cfg(all(feature = "selected", {target_flag}))]"#),
        )]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_discharges_any_feature_or_target_cfg_root_hazards() {
        let production = parse_options(["--production", "workspace", "out"]);
        let target_flag = if cfg!(unix) { "unix" } else { "windows" };
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            &format!(r#"#[cfg(any(feature = "selected", {target_flag}))]"#),
        )]);

        assert!(!production_readiness_blocks_validation(
            &production,
            &report
        ));
    }

    #[test]
    fn production_preset_keeps_non_host_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let cfg = if cfg!(unix) {
            "#[cfg(windows)]"
        } else {
            "#[cfg(unix)]"
        };
        let report = production_report(vec![target_cfg_root_hazard("app", cfg)]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_negated_custom_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            "#[cfg(not(custom_platform))]",
        )]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_negated_unproven_feature_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            r#"#[cfg(not(feature = "selected"))]"#,
        )]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_mismatched_target_cfg_root_hazards_fail_closed() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--target=x86_64-unknown-linux-gnu",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![target_cfg_root_hazard(
            "app",
            r#"#[cfg(target_arch = "wasm32")]"#,
        )]);

        assert!(production_readiness_blocks_validation(&production, &report));
    }

    #[test]
    fn production_preset_keeps_uncovered_cfg_root_hazards_fail_closed() {
        let production = parse_options(["--production", "workspace", "out"]);
        let missing_args = production_report(vec![feature_cfg_root_hazard("app", "selected")]);
        let unsupported_cfg = production_report(vec![opensource_core::ProductionHazardReport {
            code: "cfg_gated_roots".to_string(),
            severity: "error".to_string(),
            message: "root is cfg gated".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: "app::entry".to_string(),
                package: Some("app".to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some("#[cfg(custom_platform)]".to_string()),
                suggested_cargo_args: Vec::new(),
                blocked_idents: Vec::new(),
            }],
        }]);

        assert!(production_readiness_blocks_validation(
            &production,
            &missing_args
        ));
        assert!(production_readiness_blocks_validation(
            &production,
            &unsupported_cfg
        ));
    }

    #[test]
    fn production_matrix_plans_uncovered_feature_cfg_surfaces() {
        let production = parse_options(["--production", "workspace", "out"]);
        let report = production_report(vec![conditional_cfg_hazard("app", "extra")]);

        let entries = production_validation_matrix_entries(&production, &report);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "features:app/extra");
        assert!(entries[0]
            .cargo_args
            .ends_with(&["--features".to_string(), "app/extra".to_string()]));
    }

    #[test]
    fn production_matrix_skips_cfg_surfaces_covered_by_user_args() {
        let production = parse_options([
            "--production",
            "--cargo-check-arg",
            "--features",
            "--cargo-check-arg",
            "app/extra",
            "workspace",
            "out",
        ]);
        let all_features = parse_options([
            "--production",
            "--cargo-check-arg",
            "--all-features",
            "workspace",
            "out",
        ]);
        let report = production_report(vec![conditional_cfg_hazard("app", "extra")]);

        assert!(production_validation_matrix_entries(&production, &report).is_empty());
        assert!(production_validation_matrix_entries(&all_features, &report).is_empty());
    }

    #[test]
    fn production_preset_marks_warning_hazards_as_review_required_after_feedback_accepts() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "requires_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "review_required");
    }

    #[test]
    fn production_preset_accepts_feedback_discharged_warning_hazards() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        let mut readiness = gate("production_readiness", "requires_feedback");
        readiness.review_warning_hazards = Some(0);
        validation.gates.push(readiness);
        validation.gates.push(gate("feedback-repair", "accepted"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn production_preset_accepts_final_readiness_only_without_remaining_hazards() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "ready_for_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn production_preset_final_readiness_fails_after_widened_error_hazards() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation
            .gates
            .push(gate("production_readiness", "hazards_detected"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "failed");
    }

    #[test]
    fn production_preset_final_readiness_tracks_matrix_baseline_limits() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "requires_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation
            .gates
            .push(gate("production_matrix", "baseline_limited"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "baseline_limited");
    }

    #[test]
    fn production_preset_final_readiness_requires_matrix_acceptance() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "requires_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation.gates.push(gate("production_matrix", "failed"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "failed");
    }

    #[test]
    fn production_preset_final_readiness_accepts_unneeded_matrix() {
        let options = parse_options(["--production", "workspace", "out"]);
        let mut validation = ValidationReport::new(&options);
        validation
            .gates
            .push(gate("production_readiness", "ready_for_feedback"));
        validation.gates.push(gate("feedback-repair", "accepted"));
        validation
            .gates
            .push(gate("production_matrix", "not_required"));

        record_final_production_readiness(&options, &mut validation);

        let gate = validation
            .gates
            .iter()
            .find(|gate| gate.name == "production_ready")
            .expect("production_ready gate should be recorded");
        assert_eq!(gate.status, "accepted");
    }

    #[test]
    fn positional_cargo_manifest_uses_parent_as_workspace_root() {
        let nested = parse_options(["repo/rust/Cargo.toml", "out"]);
        let current_dir = parse_options(["Cargo.toml", "out"]);

        assert_eq!(nested.workspace_root, PathBuf::from("repo/rust"));
        assert_eq!(current_dir.workspace_root, PathBuf::from("."));
    }

    #[test]
    fn explicit_feedback_repair_loop_can_raise_production_preset() {
        let options = parse_options([
            "--production",
            "--feedback-repair-loop",
            "5",
            "workspace",
            "out",
        ]);

        assert_eq!(options.feedback_repair_iterations, 5);
        assert!(options.run_baseline_check);
        assert!(options.deny_warnings);
    }

    #[test]
    fn explicit_slice_report_overrides_production_default() {
        let options = parse_options([
            "--production",
            "--slice-report",
            "custom.json",
            "workspace",
            "out",
        ]);

        assert_eq!(
            slice_report_path(&options),
            Some(PathBuf::from("custom.json"))
        );
    }

    #[test]
    fn explicit_decision_log_enables_decision_report() {
        let options = parse_options(["--decision-log", "decisions.json", "workspace", "out"]);

        assert_eq!(
            decision_log_path(&options),
            Some(PathBuf::from("decisions.json"))
        );
    }

    #[test]
    fn explicit_event_log_enables_runtime_events() {
        let options = parse_options(["--event-log", "events.jsonl", "workspace", "out"]);

        assert_eq!(
            event_log_path(&options),
            Some(PathBuf::from("events.jsonl"))
        );
    }

    #[test]
    fn batch_roots_default_to_batch_event_log() {
        let options = parse_options([
            "--batch-roots",
            "--root",
            "app::selected",
            "workspace",
            "out",
        ]);

        assert_eq!(
            event_log_path(&options),
            Some(PathBuf::from("out/batch-events.jsonl"))
        );
    }

    #[test]
    fn explicit_event_log_overrides_batch_default() {
        let options = parse_options([
            "--event-log",
            "events.jsonl",
            "--batch-roots",
            "--root",
            "app::selected",
            "workspace",
            "out",
        ]);

        assert_eq!(
            event_log_path(&options),
            Some(PathBuf::from("events.jsonl"))
        );
    }

    #[test]
    fn explicit_root_options_select_roots_without_source_markers() {
        let options = parse_options([
            "--analyzer",
            "syn",
            "--root",
            "app::selected",
            "--batch-roots",
            "--random-root-package",
            "codex-ipc",
            "--random-seed",
            "7",
            "workspace",
            "out",
        ]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
        assert_eq!(options.root_selectors, ["app::selected"]);
        assert_eq!(options.random_root_packages, ["codex-ipc"]);
        assert!(options.batch_roots);
        assert_eq!(options.random_seed, 7);
    }

    #[test]
    fn batch_feedback_attempts_allow_final_widen_verification() {
        let check_only = parse_options([
            "--batch-roots",
            "--root",
            "app::selected",
            "--check",
            "workspace",
            "out",
        ]);
        assert_eq!(batch_root_attempts(&check_only), 1);

        let repair_once = parse_options([
            "--batch-roots",
            "--root",
            "app::selected",
            "--feedback-repair-loop",
            "1",
            "workspace",
            "out",
        ]);
        assert_eq!(batch_root_attempts(&repair_once), 2);

        let feedback_three = parse_options([
            "--batch-roots",
            "--root",
            "app::selected",
            "--feedback-loop",
            "3",
            "workspace",
            "out",
        ]);
        assert_eq!(batch_root_attempts(&feedback_three), 4);
    }

    #[test]
    fn named_cli_aliases_select_roots_and_output_without_positionals() {
        let options = parse_options([
            "--analyzer",
            "syn",
            "--root-selector",
            "app::selected",
            "--workspace-root",
            "workspace",
            "--output",
            "out",
        ]);

        assert_eq!(options.analyzer_mode, AnalyzerMode::Syn);
        assert_eq!(options.root_selectors, ["app::selected"]);
        assert_eq!(options.workspace_root, PathBuf::from("workspace"));
        assert_eq!(options.output_root, PathBuf::from("out"));
    }

    #[test]
    fn batch_roots_generate_without_mutating_source_markers() {
        let source = temp_path("cli-rootless-batch-source");
        let output = temp_path("cli-rootless-batch-output");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("app/src/lib.rs"),
            r#"pub mod api {
    pub fn selected() -> usize {
        helper()
    }

    pub fn helper() -> usize {
        1
    }

    pub fn dead() -> usize {
        0
    }
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--analyzer"),
            OsString::from("syn"),
            OsString::from("--batch-roots"),
            OsString::from("--root"),
            OsString::from("app::api::selected"),
            OsString::from("--preflight"),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");

        run_batch_roots(&options).expect("rootless batch should generate");

        let generated = fs::read_to_string(output.join("0001-app_api_selected/app/src/lib.rs"))
            .expect("generated source should exist");
        assert!(generated.contains("pub fn selected"), "{generated}");
        assert!(generated.contains("pub fn helper"), "{generated}");
        assert!(!generated.contains("pub fn dead"), "{generated}");
        let original = fs::read_to_string(source.join("app/src/lib.rs")).unwrap();
        assert!(!original.contains("opensourced"), "{original}");
        let report = fs::read_to_string(output.join("batch-report.jsonl")).unwrap();
        assert!(report.contains("\"status\":\"generated\""), "{report}");
        assert!(
            report.contains("\"usage_semantic_proof_status\":\"semantic_usage_unavailable\""),
            "{report}"
        );
        assert!(
            report.contains("\"usage_semantic_unproven_callables\":1"),
            "{report}"
        );
        assert!(
            report.contains("\"usage_semantic_unproven_items\":0"),
            "{report}"
        );
        let batch_events = fs::read_to_string(output.join("batch-events.jsonl")).unwrap();
        assert!(
            batch_events.contains("\"event\":\"session_manifest\""),
            "{batch_events}"
        );
        assert!(
            batch_events.contains("\"event\":\"session_parse\""),
            "{batch_events}"
        );
        assert!(
            batch_events.contains("\"event\":\"batch_analyzer\""),
            "{batch_events}"
        );
        assert!(
            batch_events.contains("\"event\":\"batch_root_context\""),
            "{batch_events}"
        );
        let root_output = output.join("0001-app_api_selected");
        let decision_log = fs::read_to_string(root_output.join("slice-decision-log.json")).unwrap();
        assert!(
            decision_log.contains("\"step\": \"top_down_closure\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"step\": \"file_retention\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"step\": \"dependency_retention\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("package app [root]"),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"step\": \"usage_pruning\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"step\": \"semantic_usage_proof\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"source_file_pruned_callables\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"step\": \"member_pruning\""),
            "{decision_log}"
        );
        let validation = fs::read_to_string(root_output.join("slice-validation.json")).unwrap();
        assert!(
            validation.contains("\"rendered_usage_contract\""),
            "{validation}"
        );
        let events = fs::read_to_string(root_output.join("slice-events.jsonl")).unwrap();
        assert!(
            events.contains("\"event\":\"batch_root_context\""),
            "{events}"
        );
        assert!(
            events.contains("\"event\":\"batch_generation\""),
            "{events}"
        );
        assert!(events.contains("\"rendered_usage\""), "{events}");
        assert!(events.contains("\"production_hazards\""), "{events}");
        assert!(events.contains("\"event\":\"batch_preflight\""), "{events}");
        assert!(events.contains("\"event\":\"batch_gate\""), "{events}");
    }

    #[test]
    fn batch_roots_preserve_explicit_selector_order() {
        let source = temp_path("cli-rootless-batch-order-source");
        let output = temp_path("cli-rootless-batch-order-output");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("app/src/lib.rs"),
            r#"pub fn selected() -> usize {
    helper()
}

pub fn helper() -> usize {
    1
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--analyzer"),
            OsString::from("syn"),
            OsString::from("--batch-roots"),
            OsString::from("--root"),
            OsString::from("app::helper"),
            OsString::from("--root"),
            OsString::from("app::selected"),
            OsString::from("--root"),
            OsString::from("app::helper"),
            OsString::from("--preflight"),
            source.into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");

        run_batch_roots(&options).expect("rootless batch should generate");

        assert!(output.join("0001-app_helper").exists());
        assert!(output.join("0002-app_selected").exists());
        assert!(!output.join("0003-app_helper").exists());
        let report = fs::read_to_string(output.join("batch-report.jsonl")).unwrap();
        let first = report.find("\"root\":\"app::helper\"").unwrap();
        let second = report.find("\"root\":\"app::selected\"").unwrap();
        assert!(
            first < second,
            "batch report should preserve explicit root order\n{report}"
        );
    }

    #[test]
    fn batch_roots_run_check_writes_per_root_feedback_artifacts() {
        let source = temp_path("cli-rootless-batch-check-source");
        let output = temp_path("cli-rootless-batch-check-output");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("app/src/lib.rs"),
            r#"pub fn selected() -> usize {
    helper()
}

pub fn helper() -> usize {
    1
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--analyzer"),
            OsString::from("syn"),
            OsString::from("--batch-roots"),
            OsString::from("--root"),
            OsString::from("app::selected"),
            OsString::from("--check"),
            source.into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");

        run_batch_roots(&options).expect("rootless batch should check");

        let root_output = output.join("0001-app_selected");
        let report = fs::read_to_string(output.join("batch-report.jsonl")).unwrap();
        assert!(report.contains("\"status\":\"accepted\""), "{report}");
        assert!(report.contains("\"feedback_diagnostics\":0"), "{report}");
        assert!(
            report.contains("\"feedback_widening_candidates\":0"),
            "{report}"
        );
        assert!(
            report.contains("\"glob_import_missing_export_candidates\":0"),
            "{report}"
        );
        assert!(
            report.contains("\"feedback_baseline_compared\":false"),
            "{report}"
        );
        assert!(
            report.contains("\"feedback_baseline_known_errors\":0"),
            "{report}"
        );
        assert!(
            report.contains("\"feedback_baseline_new_errors\":0"),
            "{report}"
        );
        assert!(root_output.join("slice-feedback.json").exists());
        let decision_log = fs::read_to_string(root_output.join("slice-decision-log.json")).unwrap();
        assert!(
            decision_log.contains("\"feedback_report\":"),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("slice-feedback.json"),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"step\": \"feedback_diagnostics\""),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("\"feedback_baseline_compared\": false"),
            "{decision_log}"
        );
        assert!(
            decision_log.contains("source baseline compared=false known_errors=0 new_errors=0"),
            "{decision_log}"
        );
        let validation = fs::read_to_string(root_output.join("slice-validation.json")).unwrap();
        assert!(
            validation.contains("\"feedback_baseline_compared\": false"),
            "{validation}"
        );
        let events = fs::read_to_string(root_output.join("slice-events.jsonl")).unwrap();
        assert!(events.contains("\"event\":\"batch_check\""), "{events}");
        assert!(
            events.contains("\"event\":\"batch_root_context\""),
            "{events}"
        );
        assert!(
            events.contains("\"event\":\"feedback_diagnostics\""),
            "{events}"
        );
        assert!(
            events.contains("\"feedback_baseline_compared\":false"),
            "{events}"
        );
        assert!(events.contains("\"widening_candidates\":0"), "{events}");
    }

    #[test]
    fn explicit_validation_report_overrides_default() {
        let options = parse_options([
            "--production",
            "--validation-report",
            "validation.json",
            "workspace",
            "out",
        ]);

        assert_eq!(
            validation_report_path(&options),
            Some(PathBuf::from("validation.json"))
        );
    }

    #[test]
    fn non_default_targets_require_matching_validation_args() {
        let targets = vec![
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "demo".to_string(),
                kind: vec!["example".to_string()],
                src_path: PathBuf::from("/workspace/app/examples/demo.rs"),
                required_features: Vec::new(),
                default_features: Vec::new(),
            },
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "behavior".to_string(),
                kind: vec!["test".to_string()],
                src_path: PathBuf::from("/workspace/app/tests/behavior.rs"),
                required_features: Vec::new(),
                default_features: Vec::new(),
            },
            GeneratedTargetReport {
                package: "app".to_string(),
                name: "throughput".to_string(),
                kind: vec!["bench".to_string()],
                src_path: PathBuf::from("/workspace/app/benches/throughput.rs"),
                required_features: Vec::new(),
                default_features: Vec::new(),
            },
        ];

        assert_eq!(
            uncovered_validation_targets(&targets, &[]),
            [
                "app example demo",
                "app test behavior",
                "app bench throughput"
            ]
        );
        assert!(uncovered_validation_targets(&targets, &["--all-targets".to_string()]).is_empty());
        assert!(uncovered_validation_targets(
            &targets,
            &[
                "--example".to_string(),
                "demo".to_string(),
                "--test=behavior".to_string(),
                "--benches".to_string()
            ]
        )
        .is_empty());
    }

    #[test]
    fn required_feature_targets_require_matching_validation_features() {
        let targets = vec![GeneratedTargetReport {
            package: "app".to_string(),
            name: "demo".to_string(),
            kind: vec!["example".to_string()],
            src_path: PathBuf::from("/workspace/app/examples/demo.rs"),
            required_features: vec!["demo-ui".to_string(), "sqlite".to_string()],
            default_features: Vec::new(),
        }];

        assert_eq!(
            uncovered_validation_targets(&targets, &["--all-targets".to_string()]),
            ["app example demo requires feature(s): demo-ui, sqlite"]
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &[
                    "--all-targets".to_string(),
                    "--features".to_string(),
                    "demo-ui,app/sqlite".to_string(),
                ]
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &["--example=demo".to_string(), "--all-features".to_string()]
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn required_feature_bin_targets_require_matching_validation_features() {
        let targets = vec![GeneratedTargetReport {
            package: "app".to_string(),
            name: "cli".to_string(),
            kind: vec!["bin".to_string()],
            src_path: PathBuf::from("/workspace/app/src/bin/cli.rs"),
            required_features: vec!["cli".to_string()],
            default_features: Vec::new(),
        }];

        assert_eq!(
            uncovered_validation_targets(&targets, &[]),
            ["app bin cli requires feature(s): cli"]
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &["--features".to_string(), "app/cli".to_string()]
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            uncovered_validation_targets(&targets, &["--all-features".to_string()]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn target_default_features_cover_required_features_unless_disabled() {
        let targets = vec![GeneratedTargetReport {
            package: "app".to_string(),
            name: "demo".to_string(),
            kind: vec!["example".to_string()],
            src_path: PathBuf::from("/workspace/app/examples/demo.rs"),
            required_features: vec!["demo-ui".to_string()],
            default_features: vec!["demo-ui".to_string()],
        }];

        assert_eq!(
            uncovered_validation_targets(&targets, &["--all-targets".to_string()]),
            Vec::<String>::new()
        );
        assert_eq!(
            uncovered_validation_targets(
                &targets,
                &[
                    "--all-targets".to_string(),
                    "--no-default-features".to_string()
                ]
            ),
            ["app example demo requires feature(s): demo-ui"]
        );
    }

    #[test]
    fn repeated_cargo_check_args_are_preserved() {
        let options = parse_options([
            "--feedback",
            "--cargo-check-arg",
            "--all-features",
            "--cargo-check-arg",
            "--target",
            "workspace",
            "out",
        ]);

        assert_eq!(options.cargo_check_args, ["--all-features", "--target"]);
    }

    #[test]
    fn feedback_widening_records_and_rerenders_from_compiler_diagnostics() {
        let source = temp_path("cli-feedback-widen-source");
        let output = temp_path("cli-feedback-widen-output");
        let slice_report = temp_path("cli-feedback-widen-report").join("slice-report.json");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}

pub fn helper() -> usize {
    2
}
"#,
        );
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--feedback-loop"),
            std::ffi::OsString::from("2"),
            std::ffi::OsString::from("--slice-report"),
            slice_report.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_helper = diagnostic("E0425", "cannot find value `helper` in this scope");
        missing_helper.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_helper]);

        let widened = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect("feedback widening should succeed");

        assert!(widened);
        assert_eq!(validation.attempts.len(), 1);
        assert_eq!(validation.attempts[0].status, "widened");
        assert_eq!(validation.attempts[0].feedback_widened_roots, Some(1));
        let generated = fs::read_to_string(output.join("app/src/lib.rs")).unwrap();
        assert!(generated.contains("pub fn helper"));
        let preserved_feedback = fs::read_to_string(output.join("slice-feedback.json")).unwrap();
        assert!(
            preserved_feedback.contains("cannot find value `helper`"),
            "{preserved_feedback}"
        );
        let report_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(slice_report).unwrap()).unwrap();
        assert_eq!(report_json["feedback_widened_roots"][0], "app::helper");
    }

    #[test]
    fn feedback_repair_loop_verifies_final_widening_before_exhausting() {
        let source = temp_path("cli-feedback-final-widen-source");
        let output = temp_path("cli-feedback-final-widen-output");
        let event_log = temp_path("cli-feedback-final-widen-events").join("events.jsonl");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}

pub fn helper() -> usize {
    2
}
"#,
        );
        write(
            output.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            output.join(".slicers-output"),
            "generated by slicers; safe to replace on the next slicers run\n",
        );
        write(
            output.join("app/Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            output.join("app/src/lib.rs"),
            r#"pub fn entry() -> usize {
    helper()
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--feedback-repair-loop"),
            OsString::from("1"),
            OsString::from("--event-log"),
            event_log.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        initialize_event_log(&options).expect("event log should initialize");
        let mut validation = ValidationReport::new(&options);

        run_feedback_repair_loop(&options, None, &mut validation)
            .expect("final-attempt widening should be verified and accepted");

        assert_eq!(validation.attempts.len(), 2);
        assert_eq!(validation.attempts[0].status, "widened");
        assert_eq!(validation.attempts[1].status, "accepted");
        assert_eq!(validation.attempts[1].attempt, 2);
        assert!(validation
            .gates
            .iter()
            .any(|gate| { gate.name == "feedback-repair" && gate.status == "accepted" }));
        let feedback_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output.join("slice-feedback.json")).unwrap())
                .unwrap();
        assert_eq!(feedback_json["success"], true);
        let events = fs::read_to_string(event_log).unwrap();
        assert!(
            events.contains("\"event\":\"feedback_widening_verification\""),
            "{events}"
        );
        assert!(
            events.contains("reports describe the current widened workspace"),
            "{events}"
        );
    }

    #[test]
    fn feedback_widening_logs_resolution_and_skips_when_no_roots_match() {
        let source = temp_path("cli-feedback-widen-no-root-source");
        let output = temp_path("cli-feedback-widen-no-root-output");
        let event_log = temp_path("cli-feedback-widen-no-root-events").join("events.jsonl");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--event-log"),
            event_log.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        initialize_event_log(&options).expect("event log should initialize");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_type = diagnostic(
            "E0425",
            "cannot find type `DefinitelyMissing` in this scope",
        );
        missing_type.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_type]);

        let widened = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect("feedback widening should resolve without rerendering");

        assert!(!widened);
        let events = fs::read_to_string(event_log).unwrap();
        assert!(events.contains("\"event\":\"feedback_widening_resolution\""));
        assert!(events.contains("\"skipped_no_match\":1"));
        assert!(events.contains("compiler feedback produced no additional roots"));
    }

    #[test]
    fn feedback_diagnostics_log_resolution_and_source_api_mismatch_candidates() {
        let source = temp_path("cli-feedback-diagnostics-source-api-source");
        let output = temp_path("cli-feedback-diagnostics-source-api-output");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"provider\"]\nresolver = \"2\"\n",
        );
        write(
			source.join("app/Cargo.toml"),
			&format!(
				"[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nprovider = {{ path = \"../provider\" }}\n",
				opensourced_path
			),
		);
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            source.join("provider/Cargo.toml"),
            "[package]\nname = \"provider\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("provider/src/lib.rs"),
            r#"pub mod types {
    pub struct Account;
}
"#,
        );
        let options = parse_args_from(vec![
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let feedback_path = output.join("slice-feedback.json");
        let mut unresolved = diagnostic_with_span(
            "E0432",
            "unresolved import `provider::types::AppAccount`",
            "app/src/lib.rs",
            3,
            5,
            32,
        );
        unresolved.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        unresolved.rendered = Some(
			"error[E0432]: unresolved import `provider::types::AppAccount`\n   |\n   | use provider::types::AppAccount;\n   |                      ---------- help: a similar name exists in the module: `Account`\n   |                      no `AppAccount` in `types`\n"
				.to_string(),
		);
        unresolved
            .suggestions
            .push(opensource_core::CheckSuggestion {
                level: "help".to_string(),
                message: "a similar name exists in the module".to_string(),
                file_name: "app/src/lib.rs".to_string(),
                line_start: 3,
                line_end: 3,
                column_start: 22,
                column_end: 32,
                byte_start: None,
                byte_end: None,
                suggested_replacement: "Account".to_string(),
                suggestion_applicability: Some("MaybeIncorrect".to_string()),
            });
        let check_report = report(false, vec![unresolved]);
        write_report(&check_report, &feedback_path).expect("feedback report should write");
        record_feedback_attempt(
            &mut validation,
            "feedback",
            1,
            "rejected",
            "generated workspace failed",
            &check_report,
            feedback_path,
            false,
            0,
            0,
            None,
            None,
        );

        let summary = decision_log_feedback_diagnostics(&options, &validation, 40);

        assert_eq!(summary.resolution_reports, 1);
        assert_eq!(summary.source_api_mismatch_candidates, 1);
        assert!(
            summary.evidence.iter().any(|entry| {
                entry.contains(
                    "source_api_mismatch_candidate unresolved_path=`provider::types::AppAccount`",
                ) && entry.contains("suggested_symbol_matches=Account")
            }),
            "{:#?}",
            summary.evidence
        );
        assert!(
            summary.evidence.iter().any(|entry| {
                entry.contains("resolution E0432 symbol=`Account`")
                    && entry.contains("package_hint=provider")
                    && entry.contains("matches=1")
            }),
            "{:#?}",
            summary.evidence
        );
    }

    #[test]
    fn feedback_diagnostics_log_glob_import_context_for_bare_missing_symbol() {
        let source = temp_path("cli-feedback-diagnostics-glob-source");
        let output = temp_path("cli-feedback-diagnostics-glob-output");
        let event_log = temp_path("cli-feedback-diagnostics-glob-events").join("events.jsonl");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"provider\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nprovider = {{ path = \"../provider\" }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use provider::conversation::*;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            source.join("provider/Cargo.toml"),
            "[package]\nname = \"provider\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("provider/src/lib.rs"),
            r#"pub mod conversation {
    pub struct Existing;
}
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--event-log"),
            event_log.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        initialize_event_log(&options).expect("event log should initialize");
        let mut validation = ValidationReport::new(&options);
        let feedback_path = output.join("slice-feedback.json");
        let mut unresolved = diagnostic_with_span(
            "E0412",
            "cannot find type `ConversationItem` in this scope",
            "app/src/lib.rs",
            5,
            22,
            38,
        );
        unresolved.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let check_report = report(false, vec![unresolved]);
        write_report(&check_report, &feedback_path).expect("feedback report should write");
        record_feedback_attempt(
            &mut validation,
            "feedback-repair",
            2,
            "rejected_after_widen",
            "final verification after compiler feedback widening did not pass",
            &check_report,
            feedback_path,
            false,
            0,
            0,
            None,
            None,
        );

        let summary = decision_log_feedback_diagnostics(&options, &validation, 40);
        write_feedback_diagnostics_event(&options, &validation)
            .expect("feedback diagnostics event should write");

        assert_eq!(summary.resolution_reports, 1);
        assert_eq!(summary.glob_import_context_entries, 1);
        assert_eq!(summary.glob_import_missing_export_candidates, 1);
        assert_eq!(summary.glob_import_private_upstream_candidates, 0);
        assert!(
            summary.evidence.iter().any(|entry| {
                entry.contains("glob_import_missing_export_candidate symbol=`ConversationItem`")
                    && entry.contains("diagnostic_file=app/src/lib.rs")
                    && entry.contains("glob_imports=provider::conversation::*")
                    && entry.contains("provider_module_roots=provider::conversation(Mod)")
            }),
            "{:#?}",
            summary.evidence
        );
        assert!(
            summary.evidence.iter().any(|entry| {
                entry.contains("symbol=`ConversationItem`")
                    && entry.contains(
                        "action=no matching project-local root; inspect glob import provider module roots",
                    )
                    && entry.contains("glob_imports=provider::conversation::*")
                    && entry.contains("glob_import_module_roots=provider::conversation(Mod)")
            }),
            "{:#?}",
            summary.evidence
        );
        let events = fs::read_to_string(event_log).unwrap();
        assert!(events.contains("\"event\":\"feedback_diagnostics\""));
        assert!(events.contains("\"glob_import_missing_export_candidates\":1"));
        assert!(events.contains("\"glob_import_private_upstream_candidates\":0"));
        assert!(events.contains("glob_import_missing_export_candidate symbol=`ConversationItem`"));
    }

    #[test]
    fn feedback_diagnostics_log_private_provider_glob_for_bare_missing_symbol() {
        let source = temp_path("cli-feedback-diagnostics-private-glob-source");
        let output = temp_path("cli-feedback-diagnostics-private-glob-output");
        let event_log =
            temp_path("cli-feedback-diagnostics-private-glob-events").join("events.jsonl");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"provider\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\nprovider = {{ path = \"../provider\" }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;
use provider::conversation::*;

#[opensourced]
pub fn entry() -> usize {
    1
}
"#,
        );
        write(
            source.join("provider/Cargo.toml"),
            "[package]\nname = \"provider\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        write(
            source.join("provider/src/lib.rs"),
            r#"pub mod conversation;
pub mod conversation_uniffi;
"#,
        );
        write(
            source.join("provider/src/conversation.rs"),
            r#"use crate::conversation_uniffi::*;

pub fn hydrate() -> HydratedConversationItem {
    HydratedConversationItem
}
"#,
        );
        write(
            source.join("provider/src/conversation_uniffi.rs"),
            r#"pub struct HydratedConversationItem;
"#,
        );
        let options = parse_args_from(vec![
            OsString::from("--event-log"),
            event_log.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        initialize_event_log(&options).expect("event log should initialize");
        let mut validation = ValidationReport::new(&options);
        let feedback_path = output.join("slice-feedback.json");
        let mut unresolved = diagnostic_with_span(
            "E0412",
            "cannot find type `ConversationItem` in this scope",
            "app/src/lib.rs",
            5,
            22,
            38,
        );
        unresolved.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let check_report = report(false, vec![unresolved]);
        write_report(&check_report, &feedback_path).expect("feedback report should write");
        record_feedback_attempt(
            &mut validation,
            "feedback-repair",
            2,
            "rejected_after_widen",
            "final verification after compiler feedback widening did not pass",
            &check_report,
            feedback_path,
            false,
            0,
            0,
            None,
            None,
        );

        let summary = decision_log_feedback_diagnostics(&options, &validation, 40);
        write_feedback_diagnostics_event(&options, &validation)
            .expect("feedback diagnostics event should write");

        assert_eq!(summary.resolution_reports, 1);
        assert_eq!(summary.glob_import_context_entries, 1);
        assert_eq!(summary.glob_import_missing_export_candidates, 0);
        assert_eq!(summary.glob_import_private_upstream_candidates, 1);
        assert!(
            summary.evidence.iter().any(|entry| {
                entry.contains("glob_import_private_upstream_candidate symbol=`ConversationItem`")
                    && entry.contains("diagnostic_file=app/src/lib.rs")
                    && entry.contains("glob_imports=provider::conversation::*")
                    && entry.contains("provider_module_roots=provider::conversation(Mod)")
                    && entry.contains("provider_internal_globs=provider::conversation_uniffi::*")
            }),
            "{:#?}",
            summary.evidence
        );
        assert!(
            summary.evidence.iter().any(|entry| {
                entry.contains("symbol=`ConversationItem`")
                    && entry.contains("glob_imports=provider::conversation::*")
                    && entry.contains("glob_import_module_roots=provider::conversation(Mod)")
                    && entry.contains("provider_internal_globs=provider::conversation_uniffi::*")
            }),
            "{:#?}",
            summary.evidence
        );
        let events = fs::read_to_string(event_log).unwrap();
        assert!(events.contains("\"event\":\"feedback_diagnostics\""));
        assert!(events.contains("\"glob_import_missing_export_candidates\":0"));
        assert!(events.contains("\"glob_import_private_upstream_candidates\":1"));
        assert!(events.contains("glob_import_private_upstream_candidate symbol=`ConversationItem`"));
        assert!(!events.contains("glob_import_missing_export_candidate symbol=`ConversationItem`"));
    }

    #[test]
    fn feedback_widening_reconciles_lockfile_before_next_locked_check() {
        let source = temp_path("cli-feedback-widen-lock-source");
        let output = temp_path("cli-feedback-widen-lock-output");
        let slice_report = temp_path("cli-feedback-widen-lock-report").join("slice-report.json");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(source.join("Cargo.lock"), stale_generated_lockfile());
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}

pub fn helper() -> usize {
    2
}
"#,
        );
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            std::ffi::OsString::from("--slice-report"),
            slice_report.into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_helper = diagnostic("E0425", "cannot find value `helper` in this scope");
        missing_helper.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_helper]);

        let widened = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect("feedback widening should reconcile lockfile");

        assert!(widened);
        let lockfile = fs::read_to_string(output.join("Cargo.lock")).unwrap();
        assert!(!lockfile.contains("dead-helper"));
        assert!(validation
            .gates
            .iter()
            .any(|gate| gate.name == "lockfile" && gate.status == "passed"));
    }

    #[test]
    fn production_feedback_widening_rechecks_readiness_and_blocks_error_hazards() {
        let source = temp_path("cli-feedback-widen-cfg-source");
        let output = temp_path("cli-feedback-widen-cfg-output");
        let slice_report = temp_path("cli-feedback-widen-cfg-report").join("slice-report.json");
        let opensourced_path = repo_root().join("crates/opensourced");
        write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
        );
        write(
            source.join("app/Cargo.toml"),
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nextra = []\n\n[dependencies]\nopensourced = {{ path = {:?} }}\n",
                opensourced_path
            ),
        );
        write(
            source.join("app/src/lib.rs"),
            r#"use opensourced::opensourced;

#[opensourced]
pub fn entry() -> usize {
    1
}

#[cfg(feature = "extra")]
pub fn helper() -> usize {
    2
}
"#,
        );
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--production"),
            std::ffi::OsString::from("--slice-report"),
            slice_report.clone().into_os_string(),
            source.clone().into_os_string(),
            output.clone().into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);
        let mut state = FeedbackWideningState::default();
        let mut missing_helper = diagnostic("E0425", "cannot find value `helper` in this scope");
        missing_helper.package_id = Some("app 0.1.0 (path+file:///tmp/app)".to_string());
        let feedback_report = report(false, vec![missing_helper]);

        let error = try_widen_from_feedback(
            &options,
            &mut validation,
            &mut state,
            "feedback-repair",
            1,
            &feedback_report,
            &output.join("slice-feedback.json"),
            0,
            0,
        )
        .expect_err("cfg-gated widened root should block production");

        assert!(error
            .to_string()
            .contains("production readiness reported error hazards"));
        assert!(validation.gates.iter().any(|gate| {
            gate.name == "production_readiness" && gate.status == "hazards_detected"
        }));
        let report_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(slice_report).unwrap()).unwrap();
        assert_eq!(report_json["production"]["status"], "hazards_detected");
        assert_eq!(report_json["feedback_widened_roots"][0], "app::helper");
    }

    #[test]
    fn plain_check_failure_writes_rejected_validation_report() {
        let source = temp_path("cli-plain-check-source");
        let output = temp_path("cli-plain-check-output");
        let validation_path = temp_path("cli-plain-check-report").join("validation.json");
        let event_log_path = temp_path("cli-plain-check-events").join("events.jsonl");
        write(output.join("Cargo.toml"), "not valid toml");
        let options = parse_args_from(vec![
            std::ffi::OsString::from("--check"),
            std::ffi::OsString::from("--validation-report"),
            validation_path.clone().into_os_string(),
            std::ffi::OsString::from("--event-log"),
            event_log_path.clone().into_os_string(),
            source.into_os_string(),
            output.into_os_string(),
        ])
        .expect("arguments should parse");
        let mut validation = ValidationReport::new(&options);

        let error = run_plain_check_gate(&options, &mut validation)
            .expect_err("plain check should fail for invalid Cargo.toml");

        assert!(error.to_string().contains("generated workspace failed"));
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(validation_path).unwrap()).unwrap();
        assert_eq!(value["status"], "rejected");
        assert_eq!(value["gates"][0]["name"], "check");
        assert_eq!(value["gates"][0]["status"], "failed");
        let events = fs::read_to_string(event_log_path).unwrap();
        assert!(events.contains(r#""event":"check""#), "{events}");
        assert!(events.contains(r#""status":"failed""#), "{events}");
        assert!(events.contains("stderr_excerpt"), "{events}");
    }

    fn report(success: bool, diagnostics: Vec<CheckDiagnostic>) -> CheckReport {
        CheckReport {
            manifest_path: PathBuf::from("/tmp/Cargo.toml"),
            working_dir: None,
            target_dir: None,
            timeout_ms: None,
            cargo_args: Vec::new(),
            success,
            timed_out: false,
            exit_code: if success { 0 } else { 101 },
            duration_ms: 25,
            diagnostics,
            widening: FeedbackWideningReport::default(),
            stderr: String::new(),
        }
    }

    fn production_report(
        hazards: Vec<opensource_core::ProductionHazardReport>,
    ) -> opensource_core::ProductionReadinessReport {
        let status = if hazards.iter().any(|hazard| hazard.severity == "error") {
            "hazards_detected"
        } else if hazards.is_empty() {
            "ready"
        } else {
            "requires_feedback"
        };
        opensource_core::ProductionReadinessReport {
            status: status.to_string(),
            hazards,
        }
    }

    fn production_hazard(code: &str, severity: &str) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: code.to_string(),
            severity: severity.to_string(),
            message: format!("{code} hazard"),
            details: Vec::new(),
        }
    }

    fn feature_cfg_root_hazard(
        package: &str,
        feature: &str,
    ) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: "cfg_gated_roots".to_string(),
            severity: "error".to_string(),
            message: "root is cfg gated".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: format!("{package}::entry"),
                package: Some(package.to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some(format!("#[cfg(feature = \"{feature}\")]")),
                suggested_cargo_args: vec!["--features".to_string(), feature.to_string()],
                blocked_idents: Vec::new(),
            }],
        }
    }

    fn target_cfg_root_hazard(package: &str, cfg: &str) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: "cfg_gated_roots".to_string(),
            severity: "error".to_string(),
            message: "root is cfg gated".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: format!("{package}::entry"),
                package: Some(package.to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some(cfg.to_string()),
                suggested_cargo_args: Vec::new(),
                blocked_idents: Vec::new(),
            }],
        }
    }

    fn conditional_cfg_hazard(
        package: &str,
        feature: &str,
    ) -> opensource_core::ProductionHazardReport {
        opensource_core::ProductionHazardReport {
            code: "conditional_compilation_attrs".to_string(),
            severity: "warning".to_string(),
            message: "retained cfg surface".to_string(),
            details: vec![opensource_core::ProductionHazardDetail {
                subject: package.to_string(),
                package: Some(package.to_string()),
                module_path: None,
                file: None,
                start_line: None,
                cfg: Some(format!("#[cfg(feature = \"{feature}\")]")),
                suggested_cargo_args: vec!["--features".to_string(), feature.to_string()],
                blocked_idents: Vec::new(),
            }],
        }
    }

    fn diagnostic(code: &str, message: &str) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn diagnostic_with_span(
        code: &str,
        message: &str,
        file_name: &str,
        line_start: u64,
        column_start: u64,
        column_end: u64,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: vec![opensource_core::CheckSpan {
                file_name: file_name.to_string(),
                line_start,
                line_end: line_start,
                column_start,
                column_end,
                is_primary: true,
                text: Vec::new(),
            }],
            suggestions: Vec::new(),
        }
    }

    fn diagnostic_with_rendered(code: &str, message: &str, rendered: &str) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: Some(rendered.to_string()),
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn warning(message: &str) -> CheckDiagnostic {
        warning_with_code("unused_variables", message)
    }

    fn warning_with_code(code: &str, message: &str) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "warning".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: None,
            target: None,
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn diagnostic_for_target(
        code: &str,
        message: &str,
        package_id: &str,
        target_kind: &str,
        target_name: &str,
    ) -> CheckDiagnostic {
        CheckDiagnostic {
            level: "error".to_string(),
            message: message.to_string(),
            code: Some(code.to_string()),
            package_id: Some(package_id.to_string()),
            target: Some(CheckTarget {
                name: target_name.to_string(),
                kind: vec![target_kind.to_string()],
                src_path: None,
            }),
            rendered: None,
            spans: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    fn stale_generated_lockfile() -> &'static str {
        r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "dead-helper",
]

[[package]]
name = "dead-helper"
version = "0.1.0"
"#
    }

    fn parse_options<const N: usize>(args: [&str; N]) -> super::CliOptions {
        parse_args_from(args.into_iter().map(std::ffi::OsString::from))
            .expect("arguments should parse")
    }

    fn gate(name: &str, status: &str) -> ValidationGateReport {
        ValidationGateReport {
            name: name.to_string(),
            status: status.to_string(),
            reason: String::new(),
            report_path: None,
            error_count: None,
            warning_count: None,
            semantic_warning_hazards: None,
            review_warning_hazards: None,
        }
    }

    fn repo_root() -> PathBuf {
        let mut current = std::env::current_dir().expect("current directory should resolve");
        loop {
            if current.join("crates/opensourced/Cargo.toml").exists() {
                return current;
            }
            assert!(
                current.pop(),
                "could not find repository root from current directory"
            );
        }
    }

    fn temp_path(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("slicers-{label}-{unique}"))
    }

    fn write(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}
