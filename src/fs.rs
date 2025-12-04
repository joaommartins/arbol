use std::fs;
use std::path::{Path, PathBuf};

use tree_sitter::{Language, Node, Parser as TsParser, Tree};
use walkdir::WalkDir;

use crate::error::{ArbolError, Result};
use crate::types::{FileAst, JsonNode};

pub fn collect_rust_files(root: &Path, include_tests: bool, skip_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut v = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if path
            .components()
            .any(|c| matches!(c.as_os_str().to_str(), Some("target") | Some("generated")))
        {
            continue;
        }
        // Custom skip directories provided by user (via CLI). A provided path may be absolute or
        // relative to root. We skip any file whose path starts with one of the skip dirs.
        if should_skip(path, root, skip_dirs) {
            continue;
        }
        if !include_tests
            && path
                .components()
                .any(|c| matches!(c.as_os_str().to_str(), Some("tests" | "benches")))
        {
            continue;
        }
        v.push(path.to_path_buf());
    }
    Ok(v)
}

fn should_skip(path: &Path, root: &Path, skip_dirs: &[PathBuf]) -> bool {
    for skip in skip_dirs {
        if skip.as_os_str().is_empty() {
            continue;
        }
        if skip.is_absolute() {
            if path.starts_with(skip) {
                return true;
            }
        } else {
            // Treat as relative to root
            let candidate = root.join(skip);
            if path.starts_with(&candidate) {
                return true;
            }
            // Also allow simple substring match on relative display to support loose patterns
            if let Ok(rel) = path.strip_prefix(root) {
                let rel_s = rel.to_string_lossy();
                let skip_s = skip.to_string_lossy();
                if rel_s.contains(skip_s.as_ref()) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn dump_file(lang: &Language, path: &Path, with_source: bool, max_depth: usize) -> Result<FileAst> {
    let src = fs::read_to_string(path)?;
    let tree = parse_src_lang(lang, &src)?;
    let root = tree.root_node();
    let mut nodes = Vec::new();
    collect_nodes(root, &src, with_source, max_depth, 0, &mut nodes);
    Ok(FileAst {
        path: path.display().to_string(),
        root_kind: root.kind().to_string(),
        nodes,
    })
}

fn parse_src_lang(lang: &Language, src: &str) -> Result<Tree> {
    let mut parser = TsParser::new();
    parser
        .set_language(lang)
        .map_err(|e| ArbolError::SetLanguage(format!("{e}")))?;
    parser.parse(src, None).ok_or(ArbolError::ParseFailed)
}

fn collect_nodes(node: Node, src: &str, with_source: bool, max_depth: usize, depth: usize, out: &mut Vec<JsonNode>) {
    if depth > max_depth {
        return;
    }
    let start = node.start_position();
    let end = node.end_position();
    let text = if with_source && node.byte_range().len() <= 240 {
        Some(node.utf8_text(src.as_bytes()).unwrap_or("").to_string())
    } else {
        None
    };
    out.push(JsonNode {
        kind: node.kind().to_string(),
        start_byte: node.start_byte() as u32,
        end_byte: node.end_byte() as u32,
        start_line: start.row as u32,
        end_line: end.row as u32,
        child_count: node.child_count() as u32,
        text,
    });
    for child in node.children(&mut node.walk()) {
        collect_nodes(child, src, with_source, max_depth, depth + 1, out);
    }
}
