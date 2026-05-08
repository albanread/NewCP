//! CLI: read a `.odc` file and emit its structural YAML projection.
//!
//! Usage:
//!   odc-yaml <input.odc> [-o output.yaml]
//!   odc-yaml --tree <input.odc>      print a tree view instead of YAML

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use newcp_odc::{document_to_yaml, read_document, StoreNode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut tree = false;

    let mut iter = args.into_iter();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--tree" => tree = true,
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

    let doc = match read_document(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let text = if tree { tree_view(&doc.root) } else { document_to_yaml(&doc) };

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

fn print_usage() {
    eprintln!(
        "odc-yaml — read BlackBox .odc files\n\
         \n\
         Usage:\n  \
           odc-yaml <input.odc> [-o output.yaml]\n  \
           odc-yaml --tree <input.odc>"
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
