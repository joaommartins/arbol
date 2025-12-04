use std::collections::HashMap;
use std::path::Path;

use rayon::prelude::*;
use tracing::warn;
use tree_sitter::{Parser as TsParser, StreamingIterator};

use crate::error::{ArbolError, Result};
use crate::fs::collect_rust_files;
use crate::types::{Capture, CrateCaptures};
use crate::workspace::{crate_for_file, discover_crate_roots};

pub fn execute_query(
    lang: &tree_sitter::Language,
    root: &Path,
    include_tests: bool,
    query_src: &str,
    context: bool,
    skip_dirs: &[std::path::PathBuf],
) -> Result<Vec<CrateCaptures>> {
    let files = collect_rust_files(root, include_tests, skip_dirs)?;
    let crate_roots = discover_crate_roots(root)?;
    let ts_query = match tree_sitter::Query::new(lang, query_src) {
        Ok(q) => q,
        Err(_) => {
            return Err(ArbolError::QueryCompile);
        }
    };
    let capture_names = ts_query.capture_names().to_vec();
    // Validate language can be set once (avoid per-thread expect/unwrap)
    {
        let mut test_parser = TsParser::new();
        test_parser
            .set_language(lang)
            .map_err(|e| ArbolError::SetLanguage(e.to_string()))?;
    }

    let captures: Vec<Capture> = files
        .par_iter()
        .map(|p| {
            let mut parser = TsParser::new();
            if let Err(e) = parser.set_language(lang) {
                warn!("tree-sitter: set_language failed for {}: {}", p.display(), e);
                return Vec::new();
            }
            let src = match std::fs::read_to_string(p) {
                Ok(s) => s,
                Err(e) => {
                    warn!("io: failed to read {}: {}", p.display(), e);
                    return Vec::new();
                }
            };
            let tree = match parser.parse(&src, None) {
                Some(t) => t,
                None => {
                    warn!("tree-sitter: parse returned None for {}", p.display());
                    return Vec::new();
                }
            };
            let mut cursor = tree_sitter::QueryCursor::new();
            let root_node = tree.root_node();
            let crate_path = crate_for_file(p, &crate_roots).unwrap_or_default();
            let lines: Vec<&str> = if context { src.lines().collect() } else { Vec::new() };
            let file_path = p.display().to_string();
            let mut out = Vec::new();
            let mut matches = cursor.matches(&ts_query, root_node, src.as_bytes());
            while let Some(m) = matches.next() {
                for cap in m.captures.iter() {
                    let node = cap.node;
                    let pos = node.start_position();
                    let text = match node.utf8_text(src.as_bytes()) {
                        Ok(t) => t.to_string(),
                        Err(e) => {
                            warn!("tree-sitter: utf8_text error in {}: {}", file_path, e);
                            String::new()
                        }
                    };
                    let line_text = if context {
                        lines.get(pos.row).copied().unwrap_or("").trim().to_string()
                    } else {
                        String::new()
                    };
                    out.push(Capture {
                        crate_path: crate_path.clone(),
                        file: file_path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        name: capture_names[cap.index as usize].to_string(),
                        text,
                        line_text,
                    });
                }
            }
            out
        })
        .reduce(Vec::new, |mut a, mut b| {
            a.append(&mut b);
            a
        });

    let mut captures = captures;
    captures.sort_by(|a, b| {
        a.crate_path
            .cmp(&b.crate_path)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.column.cmp(&b.column))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.text.cmp(&b.text))
    });

    let mut grouped: HashMap<String, Vec<Capture>> = HashMap::new();
    for c in captures {
        grouped.entry(c.crate_path.clone()).or_default().push(c);
    }

    let mut crates: Vec<CrateCaptures> = grouped
        .into_iter()
        .map(|(k, v)| CrateCaptures {
            crate_path: k,
            captures: v,
        })
        .collect();
    crates.sort_by(|a, b| a.crate_path.cmp(&b.crate_path));
    Ok(crates)
}
