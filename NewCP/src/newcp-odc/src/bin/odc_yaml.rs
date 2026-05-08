//! CLI: read a `.odc` file and emit its structural YAML projection,
//! print a structural tree, rewrite the binary back from the AST, or
//! check that read+write is byte-identical.
//!
//! Usage:
//!   odc-yaml <input.odc> [-o output.yaml]    YAML output (default)
//!   odc-yaml --tree <input.odc>              tree view of stores
//!   odc-yaml --rewrite <input.odc> -o out    re-serialize from AST
//!   odc-yaml --check <input.odc>             read, write, compare hashes

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use newcp_odc::{check_roundtrip, document_to_yaml, read_document, write_document, StoreNode};

#[derive(Copy, Clone)]
enum Mode {
    Yaml,
    Tree,
    Rewrite,
    Check,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut mode = Mode::Yaml;

    let mut iter = args.into_iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--tree" => mode = Mode::Tree,
            "--rewrite" => mode = Mode::Rewrite,
            "--check" => mode = Mode::Check,
            "-o" | "--output" => match iter.next() {
                Some(p) => output = Some(PathBuf::from(p)),
                None => {
                    eprintln!("error: -o requires a path");
                    return ExitCode::from(2);
                }
            },
            other if other.starts_with('-') => {
                eprintln!("error: unknown option {other}");
                return ExitCode::from(2);
            }
            other => {
                if input.is_some() {
                    eprintln!("error: more than one input path given");
                    return ExitCode::from(2);
                }
                input = Some(PathBuf::from(other));
            }
        }
    }

    let Some(path) = input else {
        print_usage();
        return ExitCode::from(2);
    };

    match mode {
        Mode::Check => return run_check(&path),
        Mode::Rewrite => return run_rewrite(&path, output.as_deref()),
        _ => {}
    }

    let doc = match read_document(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let text = match mode {
        Mode::Tree => tree_view(&doc.root),
        Mode::Yaml => document_to_yaml(&doc),
        _ => unreachable!(),
    };

    let result = match output {
        Some(p) => fs::write(&p, &text).map_err(io::Error::from),
        None => io::stdout().write_all(text.as_bytes()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error writing output: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_check(path: &std::path::Path) -> ExitCode {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    match check_roundtrip(bytes) {
        Ok((true, n, _)) => {
            println!("ok    {} ({n} bytes)", path.display());
            ExitCode::SUCCESS
        }
        Ok((false, n_in, n_out)) => {
            eprintln!(
                "MISMATCH  {}  in {n_in} bytes, out {n_out} bytes",
                path.display()
            );
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_rewrite(path: &std::path::Path, output: Option<&std::path::Path>) -> ExitCode {
    let doc = match read_document(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let bytes = match write_document(&doc) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let result = match output {
        Some(p) => fs::write(p, &bytes),
        None => io::stdout().write_all(&bytes),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error writing output: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "odc-yaml — read and write BlackBox .odc files\n\
         \n\
         Usage:\n  \
           odc-yaml <input.odc> [-o output.yaml]   YAML output (default)\n  \
           odc-yaml --tree <input.odc>             tree view of stores\n  \
           odc-yaml --rewrite <input.odc> -o out   re-serialize from AST\n  \
           odc-yaml --check <input.odc>            read+write, compare bytes"
    );
}

fn tree_view(root: &StoreNode) -> String {
    let mut out = String::new();
    out.push_str(&format_label(root));
    out.push('\n');
    let n = root.children.len();
    for (i, child) in root.children.iter().enumerate() {
        walk(&mut out, child, "", i + 1 == n);
    }
    out
}

fn walk(out: &mut String, node: &StoreNode, prefix: &str, last: bool) {
    let connector = if last { "└── " } else { "├── " };
    out.push_str(prefix);
    out.push_str(connector);
    out.push_str(&format_label(node));
    out.push('\n');

    let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
    let n = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        walk(out, child, &child_prefix, i + 1 == n);
    }
}

fn format_label(node: &StoreNode) -> String {
    let label = node.display_kind();
    let suffix = if node.kind.is_full_store() {
        format!("  ({} bytes, id {})", node.body_len, node.id)
    } else if let Some(t) = node.link_target {
        format!("  (target id {})", t)
    } else {
        String::new()
    };
    format!("{label}{suffix}")
}
