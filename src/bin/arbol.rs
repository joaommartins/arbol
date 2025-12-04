use std::path::PathBuf;

use arbol::Result;
use clap::{Parser, Subcommand};
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(
    about = "Tree-sitter based Rust workspace explorer",
    subcommand_required = false,
    arg_required_else_help = false
)]
struct Cli {
    /// Path to crate root (directory with Cargo.toml)
    #[arg(global = true, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Include tests & benches
    #[arg(long, global = true)]
    include_tests: bool,

    /// Emit tracing debug
    #[arg(long, global = true)]
    verbose: bool,

    /// One or more directory paths to skip (relative to root or absolute). Repeat flag to add multiple.
    #[arg(long, global = true, value_name = "DIR", num_args=1.., action=clap::ArgAction::Append)]
    skip_dir: Vec<PathBuf>,

    /// Emit markdown help to stdout (or to HELP.md with --help-output <path>)
    #[arg(long, global = true)]
    markdown_help: bool,

    /// Output path for markdown help (defaults to stdout if omitted)
    #[arg(long, global = true)]
    help_output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Dump a lightweight CST (structure only) for each .rs file to JSON
    DumpJson {
        #[arg(long)]
        output: Option<PathBuf>,
        /// Include the raw code span text for each node (can be large)
        #[arg(long)]
        with_source: bool,
        /// Limit depth (0 = only root)
        #[arg(long, default_value_t=usize::MAX)]
        max_depth: usize,
    },
    /// Run a raw tree-sitter query across all Rust source files and aggregate captures per crate
    Query {
        /// Path to a .scm query file (if omitted, use --expr)
        #[arg(long)]
        query_file: Option<PathBuf>,
        /// Inline query expression (alternative to --query-file)
        #[arg(long)]
        expr: Option<String>,
        /// Include the source line for each capture
        #[arg(long)]
        context: bool,
        /// Emit JSON (otherwise plain text grouped by crate)
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.markdown_help {
        let md = clap_markdown::help_markdown::<Cli>();
        if let Some(p) = &cli.help_output {
            if let Err(e) = std::fs::write(p, &md) {
                eprintln!("Failed to write help markdown: {e}");
            }
        } else {
            println!("{md}");
        }
        return Ok(());
    }
    if cli.verbose {
        let _ = tracing_subscriber::fmt::try_init();
    }

    match cli.command {
        Some(Commands::DumpJson {
            output,
            with_source,
            max_depth,
        }) => {
            let asts = arbol::fs::collect_rust_files(&cli.root, cli.include_tests, &cli.skip_dir).map(|files| {
                let lang = arbol::rust_language();
                files
                    .par_iter()
                    .map(|p| arbol::fs::dump_file(&lang, p, with_source, max_depth))
                    .filter_map(|r| r.ok())
                    .collect::<Vec<arbol::FileAst>>()
            })?;
            let json = serde_json::to_string_pretty(&asts)?;
            if let Some(out) = output {
                std::fs::write(out, json)?;
            } else if !write_line(&json) {
                return Ok(());
            }
        }
        Some(Commands::Query {
            query_file,
            expr,
            context,
            json,
        }) => {
            let query_src = if let Some(f) = query_file {
                std::fs::read_to_string(f)?
            } else if let Some(e) = expr {
                e
            } else {
                return Err(arbol::ArbolError::Cli("Provide --query-file or --expr".into()));
            };
            let lang = arbol::rust_language();
            let crates =
                arbol::query::execute_query(&lang, &cli.root, cli.include_tests, &query_src, context, &cli.skip_dir)?;
            if json {
                let pretty = serde_json::to_string_pretty(&crates)?;
                if !write_line(&pretty) {
                    return Ok(());
                }
            } else {
                for c in &crates {
                    if !write_line(&format!(
                        "== Crate: {} ==",
                        if c.crate_path.is_empty() {
                            "(root)"
                        } else {
                            &c.crate_path
                        }
                    )) {
                        return Ok(());
                    }
                    for cap in &c.captures {
                        if context {
                            if !write_line(&format!(
                                "{}:{}:{} {} {} // {}",
                                cap.file, cap.line, cap.column, cap.name, cap.text, cap.line_text
                            )) {
                                return Ok(());
                            }
                        } else if !write_line(&format!(
                            "{}:{}:{} {} {}",
                            cap.file, cap.line, cap.column, cap.name, cap.text
                        )) {
                            return Ok(());
                        }
                    }
                }
                let total: usize = crates.iter().map(|c| c.captures.len()).sum();
                let _ = write_line(&format!("-- total captures: {total}"));
            }
        }
        None => {}
    }
    Ok(())
}

fn write_line(line: &str) -> bool {
    use std::io::{self, Write};
    let mut out = io::stdout().lock();
    if let Err(e) = out.write_all(line.as_bytes())
        && e.kind() == io::ErrorKind::BrokenPipe
    {
        return false;
    }
    if let Err(e) = out.write_all(b"\n")
        && e.kind() == io::ErrorKind::BrokenPipe
    {
        return false;
    }
    true
}
