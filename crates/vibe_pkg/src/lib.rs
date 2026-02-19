use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

pub const MANIFEST_FILENAME: &str = "vibe.toml";
pub const LOCK_FILENAME: &str = "vibe.lock";
pub const LOCK_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
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

#[derive(Debug, Clone)]
struct PendingReq {
    name: String,
    req: VersionReq,
}

pub fn default_mirror_root(project_root: &Path) -> PathBuf {
    project_root.join(".yb").join("pkg").join("mirror")
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::{
        install_project, resolve_project, write_lockfile, LOCK_FILENAME, MANIFEST_FILENAME,
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
        let dir = mirror_root.join(package).join(version);
        fs::create_dir_all(&dir).expect("create package dir");
        let mut raw = String::new();
        raw.push_str("[package]\n");
        raw.push_str(&format!("name = \"{package}\"\n"));
        raw.push_str(&format!("version = \"{version}\"\n"));
        if !deps.is_empty() {
            raw.push_str("\n[dependencies]\n");
            for (dep, req) in deps {
                raw.push_str(&format!("{dep} = \"{req}\"\n"));
            }
        }
        fs::write(dir.join(MANIFEST_FILENAME), raw).expect("write package manifest");
    }
}
