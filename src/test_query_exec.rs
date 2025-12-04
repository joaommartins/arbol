use std::fs;
use std::io::Write;

use tempfile::TempDir;

use crate::{dump_json, execute_query};

fn write(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn query_captures_function_names_deterministically() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='q'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), r#"pub fn alpha(){} pub fn beta(){}"#);

    let q = "(function_item name: (identifier) @fn.name)";
    let run1 = execute_query(root, false, q, false).unwrap();
    let run2 = execute_query(root, false, q, false).unwrap();
    assert_eq!(run1, run2, "results should be deterministic");

    // Flatten captures
    let caps: Vec<_> = run1
        .iter()
        .flat_map(|c| c.captures.iter().map(|k| k.text.clone()))
        .collect();
    assert!(caps.contains(&"alpha".to_string()));
    assert!(caps.contains(&"beta".to_string()));
}

#[test]
fn dump_json_respects_max_depth() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='d'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), r#"pub fn alpha(){}"#);
    let all = dump_json(root, false, false, usize::MAX).unwrap();
    let shallow = dump_json(root, false, false, 0).unwrap();
    assert_eq!(all.len(), shallow.len(), "same number of files");
    assert!(all[0].nodes.len() >= shallow[0].nodes.len());
    assert_eq!(shallow[0].nodes.len(), 1, "depth 0 should have only root node");
}

#[test]
fn query_with_context_includes_line_text() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='ctx'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn gamma(){}\n");
    let q = "(function_item name: (identifier) @fn.name)";
    let res = execute_query(root, false, q, true).unwrap();
    let cap = &res[0].captures[0];
    assert!(!cap.line_text.is_empty(), "expected line_text when context=true");
}

#[test]
fn query_without_context_has_empty_line_text() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='noctx'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn delta(){}\n");
    let q = "(function_item name: (identifier) @fn.name)";
    let res = execute_query(root, false, q, false).unwrap();
    let cap = &res[0].captures[0];
    assert!(cap.line_text.is_empty(), "expected empty line_text when context=false");
}

#[test]
fn invalid_query_returns_error() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='bad'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn epsilon(){}\n");
    let err = execute_query(root, false, "(function_item name: (identifier) @fn.name", false).unwrap_err();
    // We expect a query compile error variant
    let msg = format!("{err:?}");
    assert!(msg.contains("QueryCompile"), "expected QueryCompile error, got {msg}");
}

#[test]
fn multi_crate_grouping() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    // crate a
    write(
        &root.join("a/Cargo.toml"),
        "[package]\nname='a'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("a/src/lib.rs"), "pub fn a_fn(){}\n");
    // crate b
    write(
        &root.join("b/Cargo.toml"),
        "[package]\nname='b'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("b/src/lib.rs"), "pub fn b_fn(){}\n");
    let q = "(function_item name: (identifier) @fn.name)";
    let res = execute_query(root, false, q, false).unwrap();
    // We expect two crate groups
    assert_eq!(res.len(), 2, "expected captures grouped by crate");
    let mut names: Vec<_> = res
        .iter()
        .flat_map(|c| c.captures.iter().map(|k| k.text.clone()))
        .collect();
    names.sort();
    assert_eq!(names, vec!["a_fn", "b_fn"]);
}
