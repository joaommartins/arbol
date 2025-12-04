use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

use crate::dump_json;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn depth_zero_returns_only_root_node() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='d0'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn alpha() {}\n");
    let full = dump_json(root, false, false, usize::MAX).unwrap();
    let shallow = dump_json(root, false, false, 0).unwrap();
    assert_eq!(full.len(), shallow.len(), "same file count");
    assert_eq!(shallow[0].nodes.len(), 1, "depth 0 should include only the root node");
}

#[test]
fn with_source_includes_short_text() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='src'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn beta() { }\n");
    let asts = dump_json(root, false, true, 4).unwrap();
    assert!(!asts.is_empty());
    let has_text = asts[0].nodes.iter().any(|n| n.text.as_ref().is_some());
    assert!(
        has_text,
        "expected at least one node to include source text under threshold"
    );
}

#[test]
fn large_source_omits_long_text() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='big'\nversion='0.0.0'\nedition='2021'\n",
    );
    // create a body well over 240 bytes
    let big_body = "x".repeat(400);
    write(
        &root.join("src/lib.rs"),
        &format!("pub fn big() {{ /*{}*/ }}\n", big_body),
    );
    let asts = dump_json(root, false, true, usize::MAX).unwrap();
    // At least one node (likely root) should have text omitted due to length
    let any_none = asts[0].nodes.iter().any(|n| n.text.is_none());
    assert!(
        any_none,
        "expected at least one node with no text because span exceeded threshold"
    );
}

#[test]
fn deterministic_multiple_runs() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='det'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn a() {} pub fn b() {} pub fn c() {}\n");
    let run1 = dump_json(root, false, false, usize::MAX).unwrap();
    let run2 = dump_json(root, false, false, usize::MAX).unwrap();
    assert_eq!(run1, run2, "dump_json output should be deterministic across runs");
    // Byte ranges should be coherent
    for file in run1 {
        for node in file.nodes.windows(2) {
            let a = &node[0];
            let b = &node[1];
            assert!(a.start_byte <= a.end_byte, "node start <= end");
            assert!(
                a.end_byte <= b.end_byte || a.start_byte <= b.start_byte,
                "non-decreasing progression"
            );
        }
    }
}
