# Command-Line Help for `arbol`

This document contains the help content for the `arbol` command-line program.

**Command Overview:**

* [`arbol`↴](#arbol)
* [`arbol dump-json`↴](#arbol-dump-json)
* [`arbol query`↴](#arbol-query)

## `arbol`

Tree-sitter based Rust workspace explorer

**Usage:** `arbol [OPTIONS] [ROOT] [COMMAND]`

###### **Subcommands:**

* `dump-json` — Dump a lightweight CST (structure only) for each .rs file to JSON
* `query` — Run a raw tree-sitter query across all Rust source files and aggregate captures per crate

###### **Arguments:**

* `<ROOT>` — Path to crate root (directory with Cargo.toml)

  Default value: `.`

###### **Options:**

* `--include-tests` — Include tests & benches
* `--verbose` — Emit tracing debug
* `--markdown-help` — Emit markdown help to stdout (or to HELP.md with --help-output <path>)
* `--help-output <HELP_OUTPUT>` — Output path for markdown help (defaults to stdout if omitted)



## `arbol dump-json`

Dump a lightweight CST (structure only) for each .rs file to JSON

**Usage:** `arbol dump-json [OPTIONS]`

###### **Options:**

* `--output <OUTPUT>`
* `--with-source` — Include the raw code span text for each node (can be large)
* `--max-depth <MAX_DEPTH>` — Limit depth (0 = only root)

  Default value: `18446744073709551615`



## `arbol query`

Run a raw tree-sitter query across all Rust source files and aggregate captures per crate

**Usage:** `arbol query [OPTIONS]`

###### **Options:**

* `--query-file <QUERY_FILE>` — Path to a .scm query file (if omitted, use --expr)
* `--expr <EXPR>` — Inline query expression (alternative to --query-file)
* `--context` — Include the source line for each capture
* `--json` — Emit JSON (otherwise plain text grouped by crate)



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
