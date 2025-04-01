use super::config::Config;
use super::manifest::Manifest;
use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn list_rust_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    if dir.exists() && dir.is_dir() {
        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let path = entry?.path();
            if path.is_file() && path.extension() == Some(OsStr::new("rs")) {
                let name = path.file_stem().unwrap().to_str().unwrap();
                files.push(name.to_string());
            }
        }
    }
    Ok(files)
}

pub fn canonicalize(mut path: &Path) -> Result<PathBuf> {
    if path == Path::new("") {
        path = Path::new(".");
    }
    dunce::canonicalize(path)
        .with_context(|| format!("Failed to canonicalize `{}`", path.display()))
}

/// Tries to find a package by the given `name` in the [workspace root] or member
/// of the given [workspace] [`Manifest`], and possibly falls back to a potential
/// manifest based on the working directory or `--manifest-path` as found by
/// [`find_manifest()`] and passed as argument to `potential_manifest`.
///
/// [workspace root]: https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package
/// [workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html#workspaces
pub fn find_package_manifest_in_workspace(
    (workspace_manifest_path, workspace_manifest): &(PathBuf, Manifest),
    (potential_manifest_path, potential_manifest): (PathBuf, Manifest),
    package_name: Option<&str>,
) -> Result<(PathBuf, Manifest)> {
    let potential_manifest_dir = potential_manifest_path.parent().unwrap();
    let workspace_manifest_dir = workspace_manifest_path.parent().unwrap();
    let package_subpath = potential_manifest_dir
        .strip_prefix(workspace_manifest_dir)
        .unwrap();

    let workspace_members = workspace_manifest.members(workspace_manifest_dir)?;
    // Make sure the found workspace includes the manifest "specified" by the user via --manifest-path or $PWD
    if workspace_manifest_path != &potential_manifest_path
        && !workspace_members.contains_key(potential_manifest_dir)
    {
        anyhow::bail!(
                    "current package believes it's in a workspace when it's not:
current:   {potential_manifest_path}
workspace: {workspace_manifest_path}

this may be fixable by adding `{package_subpath}` to the `workspace.members` array of the manifest located at: {workspace_manifest_path}
Alternatively, to keep it out of the workspace, add an empty `[workspace]` table to the package's manifest.",
                    // TODO: Parse workspace.exclude and add back "add the package to the `workspace.exclude` array, or"
                    potential_manifest_path = potential_manifest_path.display(),
                    workspace_manifest_path = workspace_manifest_path.display(),
                    package_subpath = package_subpath.display(),
                );
    }

    match package_name {
        // Any package in the workspace can be used if `-p` is used
        Some(name) => {
            // Check if the workspace manifest also contains a [package]
            if let Some(package) = &workspace_manifest.package {
                if package.name == name {
                    return Ok((
                        workspace_manifest_path.to_owned(),
                        workspace_manifest.clone(),
                    ));
                }
            }

            // Check all member packages inside the workspace
            for (_manifest_dir, (manifest_path, manifest)) in workspace_members {
                // .members() already checked for it having a package
                let package = manifest.package.as_ref().unwrap();
                if package.name == name {
                    return Ok((manifest_path, manifest));
                }
            }

            anyhow::bail!(
                "package `{}` not found in workspace `{}`",
                name,
                workspace_manifest_path.display(),
            )
        }
        // Otherwise use the manifest we just found, as long as it contains `[package]`
        None => {
            anyhow::ensure!(
                potential_manifest.package.is_some(),
                "Failed to parse manifest at `{}`: virtual manifests must be configured with `[workspace]`",
                potential_manifest_path.display(),
            );
            Ok((potential_manifest_path, potential_manifest))
        }
    }
}

/// Recursively walk up the directories until finding a `Cargo.toml`
pub fn find_manifest(path: &Path) -> Result<(PathBuf, Manifest)> {
    let path = canonicalize(path)?;
    let manifest_path = path
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .find(|manifest| manifest.exists())
        .context("Didn't find Cargo.toml.")?;

    let manifest = Manifest::parse_from_toml(&manifest_path)?;

    Ok((manifest_path, manifest))
}

/// Recursively walk up the directories until finding a `Cargo.toml`
/// that contains a `[workspace]`
pub fn find_workspace(potential_root: &Path) -> Result<Option<(PathBuf, Manifest)>> {
    let potential_root = canonicalize(potential_root)?;
    for manifest_path in potential_root
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .filter(|manifest| manifest.exists())
    {
        let manifest = Manifest::parse_from_toml(&manifest_path)?;
        if manifest.workspace.is_some() {
            return Ok(Some((manifest_path, manifest)));
        }
    }
    Ok(None)
}

/// Returns the [`target-dir`] configured in `.cargo/config.toml` or `"target"` if not set.
///
/// [`target-dir`]: https://doc.rust-lang.org/cargo/reference/config.html#buildtarget-dir
pub fn get_target_dir_name(config: Option<&Config>) -> Result<String> {
    if let Some(config) = config {
        if let Some(build) = config.build.as_ref() {
            if let Some(target_dir) = &build.target_dir {
                return Ok(target_dir.clone());
            }
        }
    }
    Ok("target".to_string())
}
