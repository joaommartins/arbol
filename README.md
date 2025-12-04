# arbol – Tree‑sitter powered Rust workspace explorer 🧬

A small utility (library + CLI) that walks a Rust workspace / crate, parses each `.rs` file with Tree‑sitter (v0.25), and lets you:

- Dump a lightweight JSON representation of the concrete syntax tree (CST-ish) per file
- Run arbitrary Tree‑sitter queries across all source files and aggregate captures by crate

It focuses on being: fast, embeddable, and predictable (deterministic output ordering).

## Features

- Parallel parsing + querying via `rayon`
- Skips common noisy dirs automatically (`target/`, `generated/`)
- User‑configurable directory skipping with repeatable `--skip-dir <path>` (relative or absolute)
- Optional inclusion of tests / benches (`--include-tests`)
- Deterministic ordering of files & captures for reproducible diffs
- Optional line context for each capture (`--context`)
- Depth‑limited JSON CST dumping (`--max-depth`)
- Optional inlining of short node source spans (`--with-source`)
- Safe stdout writing (gracefully handles broken pipe)

## Install

From crates.io:

```bash
cargo install --locked arbol
```

From the repo:

```bash
cargo install --locked --git https://github.com/joaommartins/arbol
```

After install you can run as a normal binary:

```bash
arbol --help
```

## Quick Start

Dump a shallow CST (installed binary assumed):

```bash
arbol dump-json --max-depth 2 > ast.json
```

Run an inline query and emit JSON:

```bash
arbol query --expr '(function_item name: (identifier) @fn.name)' --json > fns.json
```

Use a query file with line context:

```bash
arbol query --query-file examples/functions.scm --context
```

Include tests / benches:

```bash
arbol query --include-tests --expr '(macro_invocation macro: (identifier) @macro.name)'
```

Skip specific directories (repeat `--skip-dir` or pass multiple):

```bash
arbol query \
  --skip-dir target \
  --skip-dir openapi/generated \
  --expr '(trait_item name: (type_identifier) @trait.name)' --json
```

Verbose tracing:

```bash
arbol dump-json --verbose --max-depth 1
```

## CLI Overview

Subcommands:

### DumpJson

Dump a per‑file JSON listing of nodes (optionally including node source text):

Flags:

- `--with-source` include short node snippets (<= 240 bytes)
- `--max-depth <n>` limit traversal depth (0 = only root)
- `--output <path>` write to file instead of stdout

### Query

Run a raw Tree‑sitter query across all discovered Rust files.

Provide exactly one of:

- `--query-file <file.scm>`
- `--expr '<inline s-expression>'`

Optional flags:

- `--context` include the full source line for each capture
- `--json` emit structured JSON instead of plain grouped text

Global flags:

- `--include-tests` also scan `tests/` & `benches/`
- `--skip-dir <path>` repeatable; omit any paths under these directories
- `--verbose` enable tracing subscriber
- `--root <path>` (default `.`) – directory to scan (should contain a Cargo.toml or nested crates)
- `--markdown-help` emit Markdown help to stdout (or to file with `--help-output`)
- `--help-output <path>` path to write Markdown help (implies `--markdown-help`)

## Output Schemas

### DumpJson (array of per‑file objects)

```jsonc
[{
  "path": "src/lib.rs",
  "root_kind": "source_file",
  "nodes": [
    {
      "kind": "function_item",
      "start_byte": 120,
      "end_byte": 260,
      "start_line": 10,
      "end_line": 18,
      "child_count": 5,
      "text": "fn foo() {}" // present only with --with-source and short spans
    }
  ]
}]
```

### Query (JSON mode)

```jsonc
[{
  "crate_path": "utilities/arbol",
  "captures": [
    {
      "crate_path": "utilities/arbol",
      "file": "src/lib.rs",
      "line": 42,
      "column": 5,
      "name": "fn.name",
      "text": "rust_language",
      "line_text": "pub fn rust_language() -> Language {" // only with --context
    }
  ]
}]
```

## Writing Queries 🕵️

Queries are standard Tree‑sitter S‑expressions. Example: capture all public function names:

```scm
((function_item
   (visibility_modifier) @vis
   name: (identifier) @fn.name))
```

Capture trait names:

```scm
((trait_item name: (type_identifier) @trait.name))
```

You can combine them in one file; all captures are flattened then grouped by crate.

## Performance Notes

- Parsing & querying parallelised over files (one parser per worker thread)
- Sorting captures ensures deterministic output (stable CI diffs)
- Source text for nodes is truncated by size threshold to avoid massive JSON

## Limitations / TODO

- No incremental parsing (fresh parse each run)
- No built‑in filtering by crate patterns yet
- Query diagnostics: only basic position caret reporting
- Large monolithic queries may allocate more; consider splitting

## Tips

- Use smaller `--max-depth` for structural overviews
- Pipe into `jq` for quick ad‑hoc exploration: `... DumpJson | jq '.[] | .path, .nodes[0]'`
- For speed in huge workspaces, start without `--context` then re‑run when refining

## License

MIT

## Contributing 🤝

Small focused improvements welcome:

1. Open an issue / PR
2. Add tests / examples if changing behaviour
3. Keep output ordering deterministic

## Minimal Changelog

- 0.1.0 – Initial release: dump / query, parallel execution, deterministic output, configurable directory skips.
