use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

pub const MANIFEST_FILENAME: &str = "vibe.toml";
pub const LOCK_FILENAME: &str = "vibe.lock";
pub const LOCK_SCHEMA_VERSION: u32 = 1;
pub const REGISTRY_INDEX_FILENAME: &str = "index.toml";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Manifest {
    pub package: PackageSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Resolution {
    pub root: PackageSection,
    pub packages: Vec<ResolvedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Lockfile {
    pub version: u32,
    pub package: Vec<ResolvedPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    pub installed: usize,
    pub lock_path: PathBuf,
    pub store_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    pub package: String,
    pub version: String,
    pub published_dir: PathBuf,
    pub index_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    pub kind: String,
    pub package: String,
    pub version: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReport {
    pub scanned: usize,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeEntry {
    pub package: String,
    pub current: String,
    pub latest_compatible: String,
    pub latest_available: String,
    pub requires_manifest_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeReport {
    pub entries: Vec<UpgradeEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemverDelta {
    Patch,
    Minor,
    Major,
    Unchanged,
}

#[derive(Debug, Clone)]
struct PendingReq {
    name: String,
    req: VersionReq,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct RegistryIndex {
    #[serde(default = "registry_index_version")]
    version: u32,
    #[serde(default)]
    entry: Vec<RegistryIndexEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RegistryIndexEntry {
    name: String,
    version: String,
    source: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AuditPolicy {
    #[serde(default)]
    licenses: LicensePolicy,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LicensePolicy {
    #[serde(default)]
    deny: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AdvisoryDatabase {
    #[serde(default)]
    advisory: Vec<AdvisoryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct AdvisoryEntry {
    id: String,
    package: String,
    affected: String,
    #[serde(default)]
    severity: String,
}

pub fn default_mirror_root(project_root: &Path) -> PathBuf {
    project_root.join(".yb").join("pkg").join("mirror")
}

pub fn default_registry_root(project_root: &Path) -> PathBuf {
    project_root.join(".yb").join("pkg").join("registry")
}

fn registry_index_version() -> u32 {
    1
}

pub fn load_manifest(path: &Path) -> Result<Manifest, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read manifest `{}`: {e}", path.display()))?;
    toml::from_str::<Manifest>(&raw)
        .map_err(|e| format!("failed to parse manifest `{}`: {e}", path.display()))
}

pub fn resolve_project(project_root: &Path, mirror_root: &Path) -> Result<Resolution, String> {
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    let manifest = load_manifest(&manifest_path)?;
    let mut pending = manifest
        .dependencies
        .iter()
        .map(|(name, raw_req)| {
            parse_req(raw_req).map(|req| PendingReq {
                name: name.clone(),
                req,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    pending.sort_by(|a, b| a.name.cmp(&b.name));
    let selected = BTreeMap::<String, Version>::new();
    let selected_manifests = BTreeMap::<String, Manifest>::new();
    let Some((selected_versions, manifests)) =
        solve_recursive(mirror_root, pending, selected, selected_manifests)
    else {
        return Err("failed to resolve dependencies with deterministic backtracking".to_string());
    };

    let mut packages = manifests
        .iter()
        .map(|(name, pkg_manifest)| {
            let version = selected_versions.get(name).ok_or_else(|| {
                format!("internal resolver error: missing selected version for `{name}`")
            })?;
            let mut deps = BTreeMap::new();
            for dep_name in pkg_manifest.dependencies.keys() {
                if let Some(dep_version) = selected_versions.get(dep_name) {
                    deps.insert(dep_name.clone(), dep_version.to_string());
                }
            }
            Ok(ResolvedPackage {
                name: name.clone(),
                version: version.to_string(),
                source: "mirror".to_string(),
                dependencies: deps,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Resolution {
        root: manifest.package,
        packages,
    })
}

pub fn write_lockfile(project_root: &Path, resolution: &Resolution) -> Result<PathBuf, String> {
    let lock_path = project_root.join(LOCK_FILENAME);
    let lock = Lockfile {
        version: LOCK_SCHEMA_VERSION,
        package: resolution.packages.clone(),
    };
    let mut out = String::new();
    out.push_str(&format!("version = {}\n\n", lock.version));
    for pkg in &lock.package {
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", pkg.name));
        out.push_str(&format!("version = \"{}\"\n", pkg.version));
        out.push_str(&format!("source = \"{}\"\n", pkg.source));
        if pkg.dependencies.is_empty() {
            out.push_str("dependencies = {}\n");
        } else {
            let deps = pkg
                .dependencies
                .iter()
                .map(|(name, version)| format!("{name} = \"{version}\""))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("dependencies = {{ {deps} }}\n"));
        }
        out.push('\n');
    }
    fs::write(&lock_path, out)
        .map_err(|e| format!("failed to write lockfile `{}`: {e}", lock_path.display()))?;
    Ok(lock_path)
}

pub fn install_project(project_root: &Path, mirror_root: &Path) -> Result<InstallReport, String> {
    let resolution = resolve_project(project_root, mirror_root)?;
    let lock_path = write_lockfile(project_root, &resolution)?;
    let store_root = project_root.join(".yb").join("pkg").join("store");
    fs::create_dir_all(&store_root).map_err(|e| {
        format!(
            "failed to create store root `{}`: {e}",
            store_root.display()
        )
    })?;

    for pkg in &resolution.packages {
        let src = mirror_package_dir(mirror_root, &pkg.name, &pkg.version);
        let dst = store_root.join(&pkg.name).join(&pkg.version);
        copy_dir_recursive(&src, &dst)?;
    }

    Ok(InstallReport {
        installed: resolution.packages.len(),
        lock_path,
        store_root,
    })
}

pub fn publish_project(project_root: &Path, registry_root: &Path) -> Result<PublishReport, String> {
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    let manifest = load_manifest(&manifest_path)?;
    let version = Version::parse(&manifest.package.version).map_err(|e| {
        format!(
            "invalid package version `{}` in `{}`: {e}",
            manifest.package.version,
            manifest_path.display()
        )
    })?;
    if collect_package_source_files(project_root)?.is_empty() {
        return Err(format!(
            "package `{}` has no source files ({}) to publish",
            manifest.package.name, ".yb, .vibe"
        ));
    }
    let published_dir = registry_root
        .join(&manifest.package.name)
        .join(version.to_string());
    if published_dir.exists() {
        return Err(format!(
            "registry already contains `{}` v{} at `{}`",
            manifest.package.name,
            version,
            published_dir.display()
        ));
    }
    fs::create_dir_all(&published_dir).map_err(|e| {
        format!(
            "failed to create publish destination `{}`: {e}",
            published_dir.display()
        )
    })?;
    copy_publish_sources(project_root, &published_dir)?;

    fs::create_dir_all(registry_root).map_err(|e| {
        format!(
            "failed to create registry root `{}`: {e}",
            registry_root.display()
        )
    })?;
    let index_path = registry_root.join(REGISTRY_INDEX_FILENAME);
    let mut index = load_registry_index(&index_path)?;
    if index
        .entry
        .iter()
        .any(|entry| entry.name == manifest.package.name && entry.version == version.to_string())
    {
        return Err(format!(
            "registry index already has `{}` v{}",
            manifest.package.name, version
        ));
    }
    index.entry.push(RegistryIndexEntry {
        name: manifest.package.name.clone(),
        version: version.to_string(),
        source: "local".to_string(),
    });
    index.entry.sort_by(|a, b| {
        let by_name = a.name.cmp(&b.name);
        if by_name == std::cmp::Ordering::Equal {
            Version::parse(&a.version)
                .unwrap_or_else(|_| Version::new(0, 0, 0))
                .cmp(&Version::parse(&b.version).unwrap_or_else(|_| Version::new(0, 0, 0)))
        } else {
            by_name
        }
    });
    write_registry_index(&index_path, &index)?;

    Ok(PublishReport {
        package: manifest.package.name,
        version: version.to_string(),
        published_dir,
        index_path,
    })
}

pub fn audit_project(
    project_root: &Path,
    mirror_root: &Path,
    policy_path: Option<&Path>,
    advisory_db_path: Option<&Path>,
) -> Result<AuditReport, String> {
    let resolution = resolve_project(project_root, mirror_root)?;
    let policy = load_audit_policy(policy_path)?;
    let advisory_db = load_advisory_db(advisory_db_path)?;
    let mut findings = Vec::new();
    for pkg in &resolution.packages {
        let version = Version::parse(&pkg.version)
            .map_err(|e| format!("invalid resolved version `{}` for `{}`: {e}", pkg.version, pkg.name))?;
        let manifest = load_package_manifest(mirror_root, &pkg.name, &version)?;
        let license = manifest
            .package
            .license
            .clone()
            .unwrap_or_else(|| "UNSPECIFIED".to_string());
        if policy.licenses.deny.iter().any(|deny| deny == &license) {
            findings.push(AuditFinding {
                kind: "license".to_string(),
                package: pkg.name.clone(),
                version: pkg.version.clone(),
                detail: format!("license `{license}` is denied by policy"),
            });
        }
        for advisory in &advisory_db.advisory {
            if advisory.package != pkg.name {
                continue;
            }
            let Ok(affected_req) = VersionReq::parse(&advisory.affected) else {
                continue;
            };
            if affected_req.matches(&version) {
                let severity = if advisory.severity.trim().is_empty() {
                    "unknown"
                } else {
                    advisory.severity.as_str()
                };
                findings.push(AuditFinding {
                    kind: "vulnerability".to_string(),
                    package: pkg.name.clone(),
                    version: pkg.version.clone(),
                    detail: format!(
                        "{} matched affected range `{}` (severity={severity})",
                        advisory.id, advisory.affected
                    ),
                });
            }
        }
    }
    Ok(AuditReport {
        scanned: resolution.packages.len(),
        findings,
    })
}

pub fn upgrade_plan(project_root: &Path, mirror_root: &Path) -> Result<UpgradeReport, String> {
    let resolution = resolve_project(project_root, mirror_root)?;
    let manifest = load_manifest(&project_root.join(MANIFEST_FILENAME))?;
    let selected = resolution
        .packages
        .iter()
        .map(|pkg| (pkg.name.clone(), pkg.version.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut entries = Vec::new();
    for (name, raw_req) in &manifest.dependencies {
        let Some(current) = selected.get(name) else {
            continue;
        };
        let req = VersionReq::parse(raw_req)
            .map_err(|e| format!("invalid dependency requirement `{raw_req}` for `{name}`: {e}"))?;
        let versions = available_versions(mirror_root, name)?;
        let latest_available = versions
            .first()
            .map(ToString::to_string)
            .unwrap_or_else(|| current.clone());
        let latest_compatible = versions
            .iter()
            .find(|v| req.matches(v))
            .map(ToString::to_string)
            .unwrap_or_else(|| current.clone());
        entries.push(UpgradeEntry {
            package: name.clone(),
            current: current.clone(),
            latest_compatible: latest_compatible.clone(),
            latest_available: latest_available.clone(),
            requires_manifest_change: latest_compatible != latest_available,
        });
    }
    entries.sort_by(|a, b| a.package.cmp(&b.package));
    Ok(UpgradeReport { entries })
}

pub fn semver_delta(current: &str, next: &str) -> Result<SemverDelta, String> {
    let current = Version::parse(current)
        .map_err(|e| format!("invalid `--current` version `{current}`: {e}"))?;
    let next = Version::parse(next).map_err(|e| format!("invalid `--next` version `{next}`: {e}"))?;
    if next == current {
        return Ok(SemverDelta::Unchanged);
    }
    if next.major != current.major {
        return Ok(SemverDelta::Major);
    }
    if next.minor != current.minor {
        return Ok(SemverDelta::Minor);
    }
    Ok(SemverDelta::Patch)
}

fn solve_recursive(
    mirror_root: &Path,
    mut pending: Vec<PendingReq>,
    selected: BTreeMap<String, Version>,
    selected_manifests: BTreeMap<String, Manifest>,
) -> Option<(BTreeMap<String, Version>, BTreeMap<String, Manifest>)> {
    if pending.is_empty() {
        return Some((selected, selected_manifests));
    }
    pending.sort_by(|a, b| a.name.cmp(&b.name));
    let next = pending.remove(0);

    if let Some(existing) = selected.get(&next.name) {
        if next.req.matches(existing) {
            return solve_recursive(mirror_root, pending, selected, selected_manifests);
        }
        return None;
    }

    let candidates = available_versions(mirror_root, &next.name).ok()?;
    for candidate in candidates {
        if !next.req.matches(&candidate) {
            continue;
        }
        let manifest = load_package_manifest(mirror_root, &next.name, &candidate).ok()?;
        let mut next_selected = selected.clone();
        next_selected.insert(next.name.clone(), candidate.clone());

        let mut next_manifests = selected_manifests.clone();
        next_manifests.insert(next.name.clone(), manifest.clone());

        let mut next_pending = pending.clone();
        for (dep_name, raw_req) in &manifest.dependencies {
            let req = parse_req(raw_req).ok()?;
            next_pending.push(PendingReq {
                name: dep_name.clone(),
                req,
            });
        }

        if let Some(solution) =
            solve_recursive(mirror_root, next_pending, next_selected, next_manifests)
        {
            return Some(solution);
        }
    }
    None
}

fn parse_req(raw_req: &str) -> Result<VersionReq, String> {
    VersionReq::parse(raw_req).map_err(|e| format!("invalid version requirement `{raw_req}`: {e}"))
}

fn available_versions(mirror_root: &Path, package_name: &str) -> Result<Vec<Version>, String> {
    let package_dir = mirror_root.join(package_name);
    if !package_dir.exists() {
        return Ok(Vec::new());
    }
    let mut versions = fs::read_dir(&package_dir)
        .map_err(|e| {
            format!(
                "failed to read package mirror dir `{}`: {e}",
                package_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .and_then(|raw| Version::parse(raw).ok())
        })
        .collect::<Vec<_>>();
    versions.sort();
    versions.reverse();
    Ok(versions)
}

fn mirror_package_dir(mirror_root: &Path, name: &str, version: &str) -> PathBuf {
    mirror_root.join(name).join(version)
}

fn load_package_manifest(
    mirror_root: &Path,
    name: &str,
    version: &Version,
) -> Result<Manifest, String> {
    let manifest_path =
        mirror_package_dir(mirror_root, name, &version.to_string()).join(MANIFEST_FILENAME);
    load_manifest(&manifest_path)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Err(format!(
            "package source path does not exist: `{}`",
            src.display()
        ));
    }
    fs::create_dir_all(dst).map_err(|e| {
        format!(
            "failed to create destination directory `{}`: {e}",
            dst.display()
        )
    })?;
    for entry in fs::read_dir(src)
        .map_err(|e| format!("failed to read source directory `{}`: {e}", src.display()))?
    {
        let entry = entry.map_err(|e| format!("failed to read source directory entry: {e}"))?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|e| {
                format!(
                    "failed to copy `{}` to `{}`: {e}",
                    path.display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn load_registry_index(path: &Path) -> Result<RegistryIndex, String> {
    if !path.exists() {
        return Ok(RegistryIndex {
            version: registry_index_version(),
            entry: Vec::new(),
        });
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read registry index `{}`: {e}", path.display()))?;
    toml::from_str::<RegistryIndex>(&raw)
        .map_err(|e| format!("failed to parse registry index `{}`: {e}", path.display()))
}

fn write_registry_index(path: &Path, index: &RegistryIndex) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&format!("version = {}\n\n", index.version));
    for entry in &index.entry {
        out.push_str("[[entry]]\n");
        out.push_str(&format!("name = \"{}\"\n", entry.name));
        out.push_str(&format!("version = \"{}\"\n", entry.version));
        out.push_str(&format!("source = \"{}\"\n\n", entry.source));
    }
    fs::write(path, out)
        .map_err(|e| format!("failed to write registry index `{}`: {e}", path.display()))
}

fn collect_package_source_files(project_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    collect_package_source_files_recursive(project_root, project_root, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_package_source_files_recursive(
    root: &Path,
    current: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|e| format!("failed to read source directory `{}`: {e}", current.display()))?
    {
        let entry = entry.map_err(|e| format!("failed to read source directory entry: {e}"))?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if path.is_dir() {
            if name == ".yb" || name == ".git" || name == "target" {
                continue;
            }
            collect_package_source_files_recursive(root, &path, out)?;
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        if ext == "yb" || ext == "vibe" {
            let rel = path
                .strip_prefix(root)
                .map(Path::to_path_buf)
                .unwrap_or(path.clone());
            out.push(rel);
        }
    }
    Ok(())
}

fn copy_publish_sources(project_root: &Path, destination: &Path) -> Result<(), String> {
    for entry in fs::read_dir(project_root)
        .map_err(|e| format!("failed to read project root `{}`: {e}", project_root.display()))?
    {
        let entry = entry.map_err(|e| format!("failed to read project entry: {e}"))?;
        let src = entry.path();
        let target = destination.join(entry.file_name());
        let name = entry.file_name().to_string_lossy().to_string();
        if src.is_dir() {
            if name == ".yb" || name == ".git" || name == "target" {
                continue;
            }
            copy_dir_recursive(&src, &target)?;
            continue;
        }
        fs::copy(&src, &target).map_err(|e| {
            format!(
                "failed to copy publish file `{}` to `{}`: {e}",
                src.display(),
                target.display()
            )
        })?;
    }
    Ok(())
}

fn load_audit_policy(path: Option<&Path>) -> Result<AuditPolicy, String> {
    let Some(path) = path else {
        return Ok(AuditPolicy::default());
    };
    if !path.exists() {
        return Ok(AuditPolicy::default());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read audit policy `{}`: {e}", path.display()))?;
    toml::from_str::<AuditPolicy>(&raw)
        .map_err(|e| format!("failed to parse audit policy `{}`: {e}", path.display()))
}

fn load_advisory_db(path: Option<&Path>) -> Result<AdvisoryDatabase, String> {
    let Some(path) = path else {
        return Ok(AdvisoryDatabase::default());
    };
    if !path.exists() {
        return Ok(AdvisoryDatabase::default());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("failed to read advisory db `{}`: {e}", path.display()))?;
    toml::from_str::<AdvisoryDatabase>(&raw)
        .map_err(|e| format!("failed to parse advisory db `{}`: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::{
        audit_project, install_project, publish_project, resolve_project, semver_delta,
        upgrade_plan, write_lockfile, LOCK_FILENAME, MANIFEST_FILENAME, SemverDelta,
    };

    #[test]
    fn resolver_backtracks_to_satisfy_shared_dependency() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let mirror = dir.path().join("mirror");
        fs::create_dir_all(&project).expect("create project");
        fs::create_dir_all(&mirror).expect("create mirror");

        write_project_manifest(&project, &[("a", ">=1.0.0"), ("b", "^1.0.0")]);

        write_package_manifest(&mirror, "a", "2.0.0", &[("c", "^2.0.0")]);
        write_package_manifest(&mirror, "a", "1.0.0", &[("c", "^1.0.0")]);
        write_package_manifest(&mirror, "b", "1.0.0", &[("c", "^1.0.0")]);
        write_package_manifest(&mirror, "c", "2.0.0", &[]);
        write_package_manifest(&mirror, "c", "1.5.0", &[]);

        let resolved = resolve_project(&project, &mirror).expect("resolve graph");
        let versions = resolved
            .packages
            .iter()
            .map(|p| (p.name.clone(), p.version.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(versions.get("a").map(String::as_str), Some("1.0.0"));
        assert_eq!(versions.get("b").map(String::as_str), Some("1.0.0"));
        assert_eq!(versions.get("c").map(String::as_str), Some("1.5.0"));
    }

    #[test]
    fn resolver_reports_conflict_when_no_solution_exists() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let mirror = dir.path().join("mirror");
        fs::create_dir_all(&project).expect("create project");
        fs::create_dir_all(&mirror).expect("create mirror");

        write_project_manifest(&project, &[("a", "^2.0.0"), ("b", "^1.0.0")]);
        write_package_manifest(&mirror, "a", "2.0.0", &[("c", "^2.0.0")]);
        write_package_manifest(&mirror, "b", "1.0.0", &[("c", "^1.0.0")]);
        write_package_manifest(&mirror, "c", "1.0.0", &[]);
        write_package_manifest(&mirror, "c", "2.0.0", &[]);

        let err = resolve_project(&project, &mirror).expect_err("conflict expected");
        assert!(err.contains("failed to resolve dependencies"));
    }

    #[test]
    fn lockfile_is_deterministic_across_repeated_writes() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let mirror = dir.path().join("mirror");
        fs::create_dir_all(&project).expect("create project");
        fs::create_dir_all(&mirror).expect("create mirror");

        write_project_manifest(&project, &[("a", "^1.0.0"), ("b", "^1.0.0")]);
        write_package_manifest(&mirror, "a", "1.0.0", &[]);
        write_package_manifest(&mirror, "b", "1.0.0", &[]);

        let resolved = resolve_project(&project, &mirror).expect("resolve graph");
        let first = write_lockfile(&project, &resolved).expect("write lockfile first");
        let first_contents = fs::read_to_string(&first).expect("read first lock");
        let second = write_lockfile(&project, &resolved).expect("write lockfile second");
        let second_contents = fs::read_to_string(&second).expect("read second lock");
        assert_eq!(first, second);
        assert_eq!(first_contents, second_contents);
    }

    #[test]
    fn install_flow_uses_offline_mirror_and_writes_store() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let mirror = dir.path().join("mirror");
        fs::create_dir_all(&project).expect("create project");
        fs::create_dir_all(&mirror).expect("create mirror");

        write_project_manifest(&project, &[("a", "^1.0.0")]);
        write_package_manifest(&mirror, "a", "1.0.0", &[]);
        fs::write(
            mirror.join("a").join("1.0.0").join("lib.yb"),
            "pub answer() -> Int { 42 }\n",
        )
        .expect("write package source");

        let report = install_project(&project, &mirror).expect("install");
        assert_eq!(report.installed, 1);
        assert!(project.join(LOCK_FILENAME).exists());
        assert!(report
            .store_root
            .join("a")
            .join("1.0.0")
            .join(MANIFEST_FILENAME)
            .exists());
    }

    #[test]
    fn publish_flow_writes_registry_index_and_package_payload() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let registry = dir.path().join("registry");
        fs::create_dir_all(&project).expect("create project");
        write_project_manifest(&project, &[]);
        fs::write(project.join("lib.yb"), "pub answer() -> Int { 42 }\n").expect("write source");

        let report = publish_project(&project, &registry).expect("publish");
        assert_eq!(report.package, "root");
        assert_eq!(report.version, "0.1.0");
        assert!(report
            .published_dir
            .join(MANIFEST_FILENAME)
            .exists());
        assert!(report.published_dir.join("lib.yb").exists());
        let index = fs::read_to_string(report.index_path).expect("read index");
        assert!(index.contains("name = \"root\""));
        assert!(index.contains("version = \"0.1.0\""));

        let err = publish_project(&project, &registry).expect_err("duplicate publish should fail");
        assert!(err.contains("already contains"));
    }

    #[test]
    fn audit_reports_license_and_advisory_findings() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let mirror = dir.path().join("mirror");
        fs::create_dir_all(&project).expect("create project");
        fs::create_dir_all(&mirror).expect("create mirror");
        write_project_manifest(&project, &[("demo", "^1.0.0")]);
        write_package_manifest_with_license(&mirror, "demo", "1.2.0", &[], Some("GPL-3.0"));
        fs::write(
            project.join("audit_policy.toml"),
            "[licenses]\ndeny = [\"GPL-3.0\"]\n",
        )
        .expect("write policy");
        fs::write(
            project.join("advisories.toml"),
            "[[advisory]]\nid = \"VIBESEC-2026-0001\"\npackage = \"demo\"\naffected = \"<2.0.0\"\nseverity = \"high\"\n",
        )
        .expect("write advisory db");

        let report = audit_project(
            &project,
            &mirror,
            Some(&project.join("audit_policy.toml")),
            Some(&project.join("advisories.toml")),
        )
        .expect("audit");
        assert_eq!(report.scanned, 1);
        assert!(
            report.findings.iter().any(|f| f.kind == "license"),
            "expected license finding"
        );
        assert!(
            report.findings.iter().any(|f| f.kind == "vulnerability"),
            "expected vulnerability finding"
        );
    }

    #[test]
    fn upgrade_plan_marks_major_jump_as_manifest_change() {
        let dir = tempdir().expect("temp dir");
        let project = dir.path().join("project");
        let mirror = dir.path().join("mirror");
        fs::create_dir_all(&project).expect("create project");
        fs::create_dir_all(&mirror).expect("create mirror");
        write_project_manifest(&project, &[("demo", "^1.0.0")]);
        write_package_manifest(&mirror, "demo", "1.4.0", &[]);
        write_package_manifest(&mirror, "demo", "2.0.0", &[]);
        let plan = upgrade_plan(&project, &mirror).expect("upgrade plan");
        let entry = plan
            .entries
            .iter()
            .find(|entry| entry.package == "demo")
            .expect("demo entry");
        assert_eq!(entry.latest_compatible, "1.4.0");
        assert_eq!(entry.latest_available, "2.0.0");
        assert!(entry.requires_manifest_change);
    }

    #[test]
    fn semver_delta_classifies_common_transitions() {
        assert_eq!(
            semver_delta("1.2.3", "1.2.4").expect("patch delta"),
            SemverDelta::Patch
        );
        assert_eq!(
            semver_delta("1.2.3", "1.3.0").expect("minor delta"),
            SemverDelta::Minor
        );
        assert_eq!(
            semver_delta("1.2.3", "2.0.0").expect("major delta"),
            SemverDelta::Major
        );
        assert_eq!(
            semver_delta("1.2.3", "1.2.3").expect("unchanged delta"),
            SemverDelta::Unchanged
        );
    }

    fn write_project_manifest(project_dir: &Path, deps: &[(&str, &str)]) {
        let mut raw = String::new();
        raw.push_str("[package]\n");
        raw.push_str("name = \"root\"\n");
        raw.push_str("version = \"0.1.0\"\n");
        if !deps.is_empty() {
            raw.push_str("\n[dependencies]\n");
            for (name, req) in deps {
                raw.push_str(&format!("{name} = \"{req}\"\n"));
            }
        }
        fs::write(project_dir.join(MANIFEST_FILENAME), raw).expect("write root manifest");
    }

    fn write_package_manifest(
        mirror_root: &Path,
        package: &str,
        version: &str,
        deps: &[(&str, &str)],
    ) {
        write_package_manifest_with_license(mirror_root, package, version, deps, None);
    }

    fn write_package_manifest_with_license(
        mirror_root: &Path,
        package: &str,
        version: &str,
        deps: &[(&str, &str)],
        license: Option<&str>,
    ) {
        let dir = mirror_root.join(package).join(version);
        fs::create_dir_all(&dir).expect("create package dir");
        let mut raw = String::new();
        raw.push_str("[package]\n");
        raw.push_str(&format!("name = \"{package}\"\n"));
        raw.push_str(&format!("version = \"{version}\"\n"));
        if let Some(license) = license {
            raw.push_str(&format!("license = \"{license}\"\n"));
        }
        if !deps.is_empty() {
            raw.push_str("\n[dependencies]\n");
            for (dep, req) in deps {
                raw.push_str(&format!("{dep} = \"{req}\"\n"));
            }
        }
        fs::write(dir.join(MANIFEST_FILENAME), raw).expect("write package manifest");
    }
}
