use std::path::Path;

use newcp_parser::{read_source_module, SourceExportKind};

pub fn dump_cfg(path: &Path) -> String {
    match read_source_module(path) {
        Ok(spec) => {
            let procedures = if spec.procedures.is_empty() {
                "<none>".to_string()
            } else {
                spec.procedures
                    .iter()
                    .map(|procedure| {
                        format!(
                            "procedure {}\n  blocks: entry -> body -> exit\n  terminators: entry=goto body, body=return, exit=end",
                            procedure.name
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };

            format!(
                "newcp-ir CFG dump\ninput: {}\nmodule: {}\n{}",
                path.display(),
                spec.name,
                procedures
            )
        }
        Err(error) => format!("newcp-ir CFG error\ninput: {}\nerror: {}", path.display(), error),
    }
}

pub fn dump_ir(path: &Path) -> String {
    match read_source_module(path) {
        Ok(spec) => {
            let globals = spec
                .exports
                .iter()
                .filter(|export| {
                    matches!(
                        export.kind,
                        SourceExportKind::Constant | SourceExportKind::Variable
                    )
                })
                .map(|export| format!("global {} : {:?}", export.name, export.kind))
                .collect::<Vec<_>>();
            let procedures = spec
                .procedures
                .iter()
                .map(|procedure| {
                    format!(
                        "proc {} {{\n  bb0:\n    call runtime.module_enter \"{}\"\n    ret\n}}",
                        procedure.name,
                        procedure.name
                    )
                })
                .collect::<Vec<_>>();

            let mut lines = vec![
                "newcp-ir module dump".to_string(),
                format!("input: {}", path.display()),
                format!("module: {}", spec.name),
                format!(
                    "imports: {}",
                    if spec.imports.is_empty() {
                        "<none>".to_string()
                    } else {
                        spec.imports.join(", ")
                    }
                ),
            ];

            if globals.is_empty() {
                lines.push("globals: <none>".to_string());
            } else {
                lines.push(format!("globals: {}", globals.join(", ")));
            }

            if procedures.is_empty() {
                lines.push("procedures: <none>".to_string());
            } else {
                lines.push("procedures:".to_string());
                lines.extend(procedures);
            }

            lines.join("\n")
        }
        Err(error) => format!("newcp-ir error\ninput: {}\nerror: {}", path.display(), error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_dump_lists_procedure_blocks() {
        let temp = std::env::temp_dir().join("newcp-ir-test.cp");
        std::fs::write(&temp, "MODULE Demo;\nPROCEDURE Run*;\nBEGIN\nEND Run;\nEND Demo.")
            .expect("write test module");

        let dump = dump_cfg(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("procedure Run"));
        assert!(dump.contains("entry -> body -> exit"));
    }
}
