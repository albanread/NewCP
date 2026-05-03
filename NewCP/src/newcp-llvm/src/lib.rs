use std::path::Path;

use newcp_parser::{read_source_module, SourceExportKind};

pub fn dump_llvm(path: &Path) -> String {
    match read_source_module(path) {
        Ok(spec) => {
            let mut lines = vec![
                "; newcp-llvm module dump".to_string(),
                format!("; input: {}", path.display()),
                format!("; module: {}", spec.name),
                format!("@{}.imports = private constant [0 x ptr] zeroinitializer", spec.name),
            ];

            for export in spec.exports.iter().filter(|export| {
                matches!(
                    export.kind,
                    SourceExportKind::Constant | SourceExportKind::Variable
                )
            }) {
                lines.push(format!("@{}.{} = global i64 0", spec.name, export.name));
            }

            for procedure in &spec.procedures {
                lines.push(format!("define void @{}.{}() {{", spec.name, procedure.name));
                lines.push("entry:".to_string());
                lines.push("  ret void".to_string());
                lines.push("}".to_string());
            }

            lines.join("\n")
        }
        Err(error) => format!("newcp-llvm error\ninput: {}\nerror: {}", path.display(), error),
    }
}

pub fn dump_asm(path: &Path) -> String {
    match read_source_module(path) {
        Ok(spec) => {
            let mut lines = vec![
                format!("; newcp-llvm assembly dump for {}", spec.name),
                ".text".to_string(),
            ];

            for procedure in &spec.procedures {
                lines.push(format!("{}_{}:", spec.name, procedure.name));
                lines.push("    ret".to_string());
            }

            if spec.procedures.is_empty() {
                lines.push("; no procedures emitted".to_string());
            }

            lines.join("\n")
        }
        Err(error) => format!("newcp-llvm assembly error\ninput: {}\nerror: {}", path.display(), error),
    }
}
