//! The release rows, held together (TASK-f754268004d2, GitHub #788).
//!
//! A platform ank ships is written down in about ten places: the build matrix
//! of `release.yml`, the npm smoke rows and the publish loop beside it, the
//! platform packages under `npm/`, the wrapper that resolves one of them, the
//! wrapper's `optionalDependencies`, the assembly and version scripts under
//! `.github/scripts/`, the table `install.sh` maps `uname` onto, and the rows
//! of `install.yml`. A row added to some of them and not the others is a
//! release that builds a binary nobody can install, or an install that resolves
//! a package nobody published, and every one of those is found on a tag.
//!
//! So the lists are read out of the files and compared here, where a missing
//! row turns the suite red on the commit that forgot it. The reading is
//! textual and deliberately narrow: each list is found by the line shape the
//! file already uses, and a file that changes that shape fails loudly rather
//! than comparing an empty list with another.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    let p = root().join(rel);
    fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        .replace("\r\n", "\n")
}

fn set<I: IntoIterator<Item = S>, S: Into<String>>(items: I) -> BTreeSet<String> {
    items.into_iter().map(Into::into).collect()
}

/// The platform packages: every directory `npm/ank-*`, the wrapper excluded.
fn npm_packages() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for entry in fs::read_dir(root().join("npm")).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() && name.starts_with("ank-") {
            found.insert(name);
        }
    }
    assert!(!found.is_empty(), "no platform package under npm/");
    found
}

/// The lines of the job `name` in a workflow: from `  name:` to the next job.
fn job<'a>(file: &str, text: &'a str, name: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| *l == format!("  {name}:"))
        .unwrap_or_else(|| panic!("{file}: no job {name}"));
    let end = lines[start + 1..]
        .iter()
        .position(|l| {
            l.len() > 2
                && l.starts_with("  ")
                && !l.starts_with("   ")
                && !l.trim_start().starts_with('#')
                && l.ends_with(':')
        })
        .map_or(lines.len(), |i| start + 1 + i);
    lines[start..end].to_vec()
}

/// Every value of `key:` in the given lines, which is how a matrix row is
/// spelled in these workflows.
fn values(lines: &[&str], key: &str) -> BTreeSet<String> {
    let prefix = format!("{key}: ");
    lines
        .iter()
        .filter_map(|l| {
            let t = l.trim_start().trim_start_matches("- ");
            t.strip_prefix(&prefix).map(|v| v.trim().to_string())
        })
        .collect()
}

/// The words of the first `for p in ...; do` line found after `after`.
fn for_loop(file: &str, text: &str, after: &str) -> BTreeSet<String> {
    let from = text
        .find(after)
        .unwrap_or_else(|| panic!("{file}: {after:?} not found"));
    let line = text[from..]
        .lines()
        .find(|l| l.trim_start().starts_with("for p in "))
        .unwrap_or_else(|| panic!("{file}: no `for p in` after {after:?}"));
    let words = line.trim_start().trim_start_matches("for p in ");
    let words = words.split(';').next().unwrap();
    set(words.split_whitespace())
}

/// The targets `install.sh` maps a `uname` pair onto, from `supported_lines`.
fn install_sh_targets() -> BTreeSet<String> {
    let text = read("install.sh");
    let at = text
        .find("supported_lines() {")
        .expect("install.sh: no supported_lines");
    let body = &text[at..];
    let body = &body[..body
        .find("\nEOF")
        .expect("install.sh: supported_lines unterminated")];
    let found: BTreeSet<String> = body
        .lines()
        .skip(2)
        .filter_map(|l| l.split_whitespace().nth(2).map(str::to_string))
        .collect();
    assert!(!found.is_empty(), "install.sh: supported_lines is empty");
    found
}

/// The `case` arm of `install.sh` that maps a normalised pair onto a target.
fn install_sh_maps(pair: &str) -> Option<String> {
    let text = read("install.sh");
    text.lines().find_map(|l| {
        let t = l.trim_start();
        t.strip_prefix(&format!("{pair}) target="))
            .map(|rest| rest.split_whitespace().next().unwrap().to_string())
    })
}

/// The target `npm-assemble.sh` fills a package from.
fn assembled_target(package: &str) -> Option<String> {
    let text = read(".github/scripts/npm-assemble.sh");
    text.lines().find_map(|l| {
        l.trim_start()
            .strip_prefix(&format!("{package}) echo "))
            .map(|rest| rest.split_whitespace().next().unwrap().to_string())
    })
}

#[test]
fn every_platform_package_is_named_wherever_a_package_is_listed() {
    let packages = npm_packages();
    let scoped: BTreeSet<String> = packages.iter().map(|p| format!("@haksolot/{p}")).collect();

    // The wrapper resolves one of them, and pins all of them.
    let wrapper = read("npm/ank/bin/wrapper.js");
    let resolved: BTreeSet<String> = wrapper
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            (t.starts_with('"') && t.contains("\": \"@haksolot/"))
                .then(|| t.split('"').nth(3).unwrap().to_string())
        })
        .collect();
    assert_eq!(resolved, scoped, "npm/ank/bin/wrapper.js PACKAGES");

    let manifest = read("npm/ank/package.json");
    let pinned: BTreeSet<String> = manifest
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("\"@haksolot/ank-")
                .map(|rest| format!("@haksolot/ank-{}", rest.split('"').next().unwrap()))
        })
        .collect();
    assert_eq!(pinned, scoped, "npm/ank/package.json optionalDependencies");

    let assemble = read(".github/scripts/npm-assemble.sh");
    let default_line = assemble
        .lines()
        .find(|l| l.trim_start().starts_with("packages=(ank-"))
        .expect("npm-assemble.sh: no default package list");
    let defaults = set(default_line
        .trim_start()
        .trim_start_matches("packages=(")
        .trim_end_matches(')')
        .split_whitespace());
    assert_eq!(defaults, packages, "npm-assemble.sh default packages");
    assert_eq!(
        for_loop("npm-assemble.sh", &assemble, "cd npm/ank"),
        packages,
        "npm-assemble.sh wrapper pins"
    );

    assert_eq!(
        for_loop(
            "check-version.sh",
            &read(".github/scripts/check-version.sh"),
            "--prefix npm/ank\""
        ),
        packages,
        "check-version.sh"
    );

    let fixtures = read(".github/scripts/check-version-fixtures.sh");
    for p in &packages {
        assert!(
            fixtures.contains(&format!("  npm/{p}/package.json\n")),
            "check-version-fixtures.sh copies no npm/{p}/package.json"
        );
    }

    let release = read(".github/workflows/release.yml");
    let mut published = packages.clone();
    published.insert("ank".into());
    assert_eq!(
        for_loop("release.yml", &release, "publishing $version under $tag"),
        published,
        "release.yml publish-npm loop"
    );
    assert_eq!(
        values(&job("release.yml", &release, "npm-smoke"), "package"),
        packages,
        "release.yml npm-smoke rows"
    );
}

#[test]
fn every_package_is_filled_from_a_target_the_release_builds() {
    let release = read(".github/workflows/release.yml");
    let built = values(&job("release.yml", &release, "build"), "target");
    let smoke = job("release.yml", &release, "npm-smoke");
    let smoke_targets = values(&smoke, "target");
    for p in npm_packages() {
        let target = assembled_target(&p)
            .unwrap_or_else(|| panic!("npm-assemble.sh maps no target for {p}"));
        assert!(
            built.contains(&target),
            "{p} is filled from {target}, which release.yml does not build"
        );
        assert!(
            smoke_targets.contains(&target),
            "{p}: release.yml npm-smoke has no row for {target}"
        );
    }
}

#[test]
fn every_archive_the_release_builds_for_unix_is_one_install_sh_installs_and_install_yml_tests() {
    let release = read(".github/workflows/release.yml");
    let unix: BTreeSet<String> = values(&job("release.yml", &release, "build"), "target")
        .into_iter()
        .filter(|t| !t.contains("windows"))
        .collect();
    assert_eq!(install_sh_targets(), unix, "install.sh supported_lines");

    let install = read(".github/workflows/install.yml");
    assert_eq!(
        values(&job("install.yml", &install, "install-unix"), "target"),
        unix,
        "install.yml install-unix rows"
    );
}

/// The row #788 asks for, named rather than only counted.
#[test]
fn linux_arm64_is_built_installed_and_resolved() {
    let target = "aarch64-unknown-linux-musl";
    let release = read(".github/workflows/release.yml");
    assert!(
        values(&job("release.yml", &release, "build"), "target").contains(target),
        "release.yml builds no {target}"
    );
    assert_eq!(install_sh_maps("Linux/arm64").as_deref(), Some(target));
    assert_eq!(
        assembled_target("ank-linux-arm64-musl").as_deref(),
        Some(target),
        "npm-assemble.sh"
    );
    let wrapper = read("npm/ank/bin/wrapper.js");
    assert!(
        wrapper.contains("\"linux arm64\": \"@haksolot/ank-linux-arm64-musl\""),
        "the wrapper does not resolve linux arm64"
    );
    let manifest = read("npm/ank-linux-arm64-musl/package.json");
    assert!(manifest.contains("\"name\": \"@haksolot/ank-linux-arm64-musl\""));
    assert!(
        manifest.contains("\"arm64\""),
        "the package declares no arm64 cpu"
    );
}
