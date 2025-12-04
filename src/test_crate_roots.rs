use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

use crate::workspace::{crate_for_file, discover_crate_roots};

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn discovers_crate_roots_deepest_first() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    // outer crate
    write(
        &root.join("outer/Cargo.toml"),
        "[package]\nname='outer'\nversion='0.0.0'\nedition='2021'\n",
    );
    // nested inner crate
    write(
        &root.join("outer/inner/Cargo.toml"),
        "[package]\nname='inner'\nversion='0.0.0'\nedition='2021'\n",
    );

    let crates = discover_crate_roots(root).expect("discover");
    let display: Vec<String> = crates
        .iter()
        .map(|p| p.strip_prefix(root).unwrap().display().to_string())
        .collect();
    // Expect inner (longer path) first, then outer
    assert_eq!(
        display,
        vec!["outer/inner", "outer"],
        "deepest crate path should be first"
    );
}

#[test]
fn crate_for_file_picks_deepest_match() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("outer/Cargo.toml"),
        "[package]\nname='outer'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(
        &root.join("outer/inner/Cargo.toml"),
        "[package]\nname='inner'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("outer/inner/src/lib.rs"), "pub fn x(){}\n");

    let crates = discover_crate_roots(root).unwrap();
    let file = root.join("outer/inner/src/lib.rs");
    let found = crate_for_file(&file, &crates).unwrap();
    assert!(
        found.ends_with("outer/inner"),
        "expected deepest crate root, got {found}"
    );
}

#[test]
fn crate_for_file_none_when_outside() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("a/Cargo.toml"),
        "[package]\nname='a'\nversion='0.0.0'\nedition='2021'\n",
    );
    let outside = root.join("no_crate/file.rs");
    write(&outside, "fn main(){}\n");
    let crates = discover_crate_roots(root).unwrap();
    let res = crate_for_file(&outside, &crates);
    assert!(res.is_none(), "file outside crate roots should yield None");
}
