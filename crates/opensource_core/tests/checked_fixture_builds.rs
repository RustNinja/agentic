use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn checked_in_fixture_workspaces_cargo_check() {
    let repo = repo_root();
    let mut fixtures = vec![FixtureWorkspace {
        label: "repo-root-workspace".to_owned(),
        manifest: repo.join("Cargo.toml"),
    }];
    fixtures.extend(
        fixture_workspace_manifests(&repo)
            .into_iter()
            .map(|manifest| FixtureWorkspace {
                label: fixture_label(&repo, &manifest),
                manifest,
            }),
    );
    assert_all_fixture_manifests_covered(&repo, &fixtures);

    for fixture in fixtures {
        fixture.cargo_check();
    }
}

struct FixtureWorkspace {
    label: String,
    manifest: PathBuf,
}

impl FixtureWorkspace {
    fn cargo_check(&self) {
        let target_dir = temp_path(&format!("{}-target", self.label));
        let mut command = Command::new("cargo");
        command
            .arg("check")
            .arg("--manifest-path")
            .arg(&self.manifest)
            .arg("--workspace")
            .arg("--all-targets")
            .arg("--all-features")
            .env("CARGO_TARGET_DIR", &target_dir);

        let output = command.output().unwrap_or_else(|err| {
            panic!(
                "cargo check should start for {} at {}: {err}",
                self.label,
                self.manifest.display()
            )
        });
        assert!(
            output.status.success(),
            "fixture workspace {} did not cargo check\nmanifest: {}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            self.label,
            self.manifest.display(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

fn assert_all_fixture_manifests_covered(repo: &Path, fixtures: &[FixtureWorkspace]) {
    let covered_roots = covered_fixture_roots(repo, fixtures);

    let uncovered = cargo_manifests(&repo.join("fixtures"))
        .into_iter()
        .filter(|manifest| {
            let package_root = manifest.parent().expect("manifest should have parent");
            let package_root = canonical_existing_path(package_root);
            !covered_roots
                .iter()
                .any(|covered_root| package_root.starts_with(covered_root))
        })
        .collect::<Vec<_>>();

    assert!(
        uncovered.is_empty(),
        "fixture Cargo.toml files are not covered by checked workspaces:\n{}",
        uncovered
            .iter()
            .map(|manifest| format!("- {}", manifest.display()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn covered_fixture_roots(repo: &Path, fixtures: &[FixtureWorkspace]) -> Vec<PathBuf> {
    let mut covered = BTreeSet::new();
    let mut pending = Vec::new();

    for member_root in workspace_member_roots(&repo.join("Cargo.toml")) {
        push_covered_root(&mut covered, &mut pending, member_root);
    }

    for workspace in fixtures {
        let Some(workspace_root) = workspace.manifest.parent() else {
            continue;
        };
        push_covered_root(&mut covered, &mut pending, workspace_root.to_path_buf());
        for member_root in workspace_member_roots(&workspace.manifest) {
            push_covered_root(&mut covered, &mut pending, member_root);
        }
    }

    while let Some(root) = pending.pop() {
        for dependency_root in path_dependency_roots(&root.join("Cargo.toml")) {
            push_covered_root(&mut covered, &mut pending, dependency_root);
        }
    }

    covered.into_iter().collect()
}

fn push_covered_root(covered: &mut BTreeSet<PathBuf>, pending: &mut Vec<PathBuf>, root: PathBuf) {
    let root = canonical_existing_path(&root);
    if covered.insert(root.clone()) {
        pending.push(root);
    }
}

fn path_dependency_roots(manifest: &Path) -> Vec<PathBuf> {
    let Ok(contents) = fs::read_to_string(manifest) else {
        return Vec::new();
    };
    let Ok(value) = contents.parse::<toml::Value>() else {
        return Vec::new();
    };
    let Some(manifest_dir) = manifest.parent() else {
        return Vec::new();
    };

    let mut roots = Vec::new();
    collect_dependency_table_path_roots(value.get("dependencies"), manifest_dir, &mut roots);
    collect_dependency_table_path_roots(value.get("dev-dependencies"), manifest_dir, &mut roots);
    collect_dependency_table_path_roots(value.get("build-dependencies"), manifest_dir, &mut roots);
    if let Some(targets) = value.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            collect_dependency_table_path_roots(
                target.get("dependencies"),
                manifest_dir,
                &mut roots,
            );
            collect_dependency_table_path_roots(
                target.get("dev-dependencies"),
                manifest_dir,
                &mut roots,
            );
            collect_dependency_table_path_roots(
                target.get("build-dependencies"),
                manifest_dir,
                &mut roots,
            );
        }
    }
    roots
}

fn collect_dependency_table_path_roots(
    value: Option<&toml::Value>,
    manifest_dir: &Path,
    roots: &mut Vec<PathBuf>,
) {
    let Some(table) = value.and_then(toml::Value::as_table) else {
        return;
    };
    for dependency in table.values() {
        let Some(dependency_table) = dependency.as_table() else {
            continue;
        };
        let Some(path) = dependency_table.get("path").and_then(toml::Value::as_str) else {
            continue;
        };
        roots.push(canonical_existing_path(&manifest_dir.join(path)));
    }
}

fn canonical_existing_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn fixture_workspace_manifests(repo: &Path) -> Vec<PathBuf> {
    cargo_manifests(&repo.join("fixtures"))
        .into_iter()
        .filter(|manifest| is_workspace_manifest(manifest))
        .collect()
}

fn cargo_manifests(root: &Path) -> Vec<PathBuf> {
    let mut manifests = Vec::new();
    collect_cargo_manifests(root, &mut manifests);
    manifests.sort();
    manifests
}

fn collect_cargo_manifests(root: &Path, manifests: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|err| {
        panic!(
            "fixture directory should be readable at {}: {err}",
            root.display()
        )
    });
    for entry in entries {
        let entry = entry.expect("fixture directory entry should be readable");
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name == "target" || file_name == ".git" {
            continue;
        }
        if path.is_dir() {
            collect_cargo_manifests(&path, manifests);
        } else if file_name == "Cargo.toml" {
            manifests.push(path);
        }
    }
}

fn is_workspace_manifest(manifest: &Path) -> bool {
    let contents = fs::read_to_string(manifest).unwrap_or_else(|err| {
        panic!(
            "fixture manifest should be readable at {}: {err}",
            manifest.display()
        )
    });
    contents.lines().any(|line| line.trim() == "[workspace]")
}

fn workspace_member_roots(manifest: &Path) -> Vec<PathBuf> {
    let contents = fs::read_to_string(manifest).unwrap_or_else(|err| {
        panic!(
            "workspace manifest should be readable at {}: {err}",
            manifest.display()
        )
    });
    let workspace_root = manifest.parent().expect("manifest should have parent");
    let mut roots = Vec::new();
    let mut in_members = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
        }
        if in_members {
            roots.extend(
                quoted_strings(trimmed)
                    .into_iter()
                    .map(|member| workspace_root.join(member)),
            );
        }
        if in_members && trimmed.contains(']') {
            break;
        }
    }
    roots
}

fn quoted_strings(line: &str) -> Vec<&str> {
    let mut strings = Vec::new();
    let mut remainder = line;
    while let Some(start) = remainder.find('"') {
        let after_start = &remainder[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        strings.push(&after_start[..end]);
        remainder = &after_start[end + 1..];
    }
    strings
}

fn fixture_label(repo: &Path, manifest: &Path) -> String {
    manifest
        .strip_prefix(repo)
        .unwrap_or(manifest)
        .display()
        .to_string()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root should exist")
        .to_path_buf()
}

fn temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("slicer-{label}-{nanos}"));
    if path.exists() {
        fs::remove_dir_all(&path).expect("old temp directory should remove");
    }
    path
}
