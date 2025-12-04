use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

fn write(path: &Path, content: &str) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn cli_dump_json_depth_zero_outputs_json_array() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='cli1'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn a(){}\n");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arbol"));
    cmd.current_dir(root).args(["dump-json", "--max-depth", "0"]);
    let out = cmd.assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid json");
    assert!(v.as_array().is_some(), "top level should be array");
}

#[test]
fn cli_query_json_mode_captures_function() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='cli2'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn alpha(){} pub fn beta(){}\n");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arbol"));
    cmd.current_dir(root).args([
        "query",
        "--expr",
        "(function_item name: (identifier) @fn.name)",
        "--json",
    ]);
    let out = cmd.assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let arr = v.as_array().unwrap();
    assert!(!arr.is_empty());
    let captures = &arr[0]["captures"];
    assert!(captures.to_string().contains("alpha"));
}

#[test]
fn cli_query_plain_text_contains_crate_header() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='cli3'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn zeta(){}\n");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arbol"));
    cmd.current_dir(root)
        .args(["query", "--expr", "(function_item name: (identifier) @fn.name)"]);
    cmd.assert().success().stdout(contains("== Crate:"));
}

#[test]
fn cli_respects_skip_dir_flag() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='cli_skip'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn a(){}\n");
    write(&root.join("src/generated/auto.rs"), "pub fn auto(){}\n");
    // Query for any function name; skipping src/generated should hide auto()
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arbol"));
    cmd.current_dir(root).args([
        "query",
        "--expr",
        "(function_item name: (identifier) @fn.name)",
        "--json",
        "--skip-dir",
        "src/generated",
    ]);
    let out = cmd.assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let text = v.to_string();
    assert!(text.contains("a"), "expected function a present");
    assert!(!text.contains("auto"), "expected skipped function auto not present");
}
