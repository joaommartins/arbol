pub mod error;
pub mod fs;
pub mod query;
pub mod types;
pub mod workspace;

#[cfg(test)]
mod test_crate_roots;
#[cfg(test)]
mod test_dump_json;
#[cfg(test)]
mod test_fs_collect;
#[cfg(test)]
mod test_query_exec;

use std::path::Path;

use rayon::prelude::*;
use tree_sitter::{Language, QueryError};
use tree_sitter_rust::LANGUAGE as RUST_LANGUAGE;

pub use self::error::{ArbolError, Result};
pub use self::types::*;

pub fn rust_language() -> Language {
    Language::new(RUST_LANGUAGE)
}

pub fn print_query_diagnostic(query: &str, err: &QueryError) {
    eprintln!("Query compile error: {err}");
    let err_str = err.to_string();
    if let Some(pos_idx) = err_str.rfind("position ")
        && let Some(num_str) = err_str[pos_idx + 9..].split(|c: char| !c.is_ascii_digit()).next()
        && let Ok(offset) = num_str.parse::<usize>()
        && offset < query.len()
    {
        let snippet_radius = 40usize;
        let start = offset.saturating_sub(snippet_radius);
        let end = (offset + snippet_radius).min(query.len());
        let snippet = &query[start..end];
        let caret_pos = offset - start;
        eprintln!("--- query context ---\n{}\n{}^", snippet, " ".repeat(caret_pos));
    }
}

pub fn dump_json(root: &Path, include_tests: bool, with_source: bool, max_depth: usize) -> Result<Vec<FileAst>> {
    let lang = rust_language();
    let files = fs::collect_rust_files(root, include_tests, &[])?;
    let out: Vec<FileAst> = files
        .par_iter()
        .map(|p| fs::dump_file(&lang, p, with_source, max_depth))
        .filter_map(|r| r.ok())
        .collect();
    Ok(out)
}

pub fn dump_json_with_skips(
    root: &Path,
    include_tests: bool,
    with_source: bool,
    max_depth: usize,
    skip_dirs: &[std::path::PathBuf],
) -> Result<Vec<FileAst>> {
    let lang = rust_language();
    let files = fs::collect_rust_files(root, include_tests, skip_dirs)?;
    let out: Vec<FileAst> = files
        .par_iter()
        .map(|p| fs::dump_file(&lang, p, with_source, max_depth))
        .filter_map(|r| r.ok())
        .collect();
    Ok(out)
}

pub fn execute_query(root: &Path, include_tests: bool, query_src: &str, context: bool) -> Result<Vec<CrateCaptures>> {
    let lang = rust_language();
    query::execute_query(&lang, root, include_tests, query_src, context, &[])
}

pub fn execute_query_with_skips(
    root: &Path,
    include_tests: bool,
    query_src: &str,
    context: bool,
    skip_dirs: &[std::path::PathBuf],
) -> Result<Vec<CrateCaptures>> {
    let lang = rust_language();
    query::execute_query(&lang, root, include_tests, query_src, context, skip_dirs)
}
