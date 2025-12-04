use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;

use crate::fs::collect_rust_files;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn collects_basic_rs_files_and_skips_ignored_and_generated() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    // minimal crate layout
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn a(){}\n");
    write(&root.join("src/ignored.rs"), "pub fn ignored(){}\n");
    write(&root.join("tests/test_thing.rs"), "#[test] fn t(){}\n");
    write(&root.join("target/will_be_skipped.rs"), "pub fn b(){}\n");
    write(&root.join("generated/gen.rs"), "pub fn c(){}\n");
    write(&root.join("openapi/generated/api.rs"), "pub fn d(){}\n");
    // .gitignore to exclude one file
    write(&root.join(".gitignore"), "src/ignored.rs\n");

    let without_tests = collect_rust_files(root, false, &[]).unwrap();
    let with_tests = collect_rust_files(root, true, &[]).unwrap();

    // Should contain only src/lib.rs without tests
    let mut without: Vec<String> = without_tests
        .iter()
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
        .collect();
    without.sort();
    assert_eq!(
        without,
        vec!["src/ignored.rs", "src/lib.rs"],
        "no gitignore filtering: both files present"
    );

    // With tests includes tests/test_thing.rs
    let mut rels: Vec<_> = with_tests
        .iter()
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
        .collect();
    rels.sort();
    assert_eq!(
        rels,
        vec!["src/ignored.rs", "src/lib.rs", "tests/test_thing.rs"],
        "no gitignore filtering: tests flag adds tests plus all src files"
    );
}

#[test]
fn collects_with_skip_dirs_excludes_specified_paths() {
    let td = TempDir::new().unwrap();
    let root = td.path();
    write(
        &root.join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.0.0'\nedition='2021'\n",
    );
    write(&root.join("src/lib.rs"), "pub fn a(){}\n");
    write(&root.join("src/skip_me/mod.rs"), "pub fn hidden(){}\n");
    write(&root.join("examples/demo.rs"), "pub fn demo(){}\n");
    // Provide skip dirs relative and absolute
    let abs_skip = root.join("examples");
    let res = collect_rust_files(root, false, &[Path::new("src/skip_me").to_path_buf(), abs_skip]).unwrap();
    let mut rels: Vec<_> = res
        .iter()
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
        .collect();
    rels.sort();
    assert_eq!(rels, vec!["src/lib.rs"], "expected only lib.rs after skips");
}
