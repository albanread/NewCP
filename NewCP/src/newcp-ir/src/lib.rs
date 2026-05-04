mod types;
mod ir;
mod procedure;
mod lower;

pub use types::{IrType, RecordLayout};
pub use ir::{BasicBlock, BinOp, BlockId, Instr, IrValue, TempId, Terminator, TrapKind, UnOp};
pub use procedure::{IrGlobal, IrModule, IrProcedure};
pub use lower::{lower_module, lower_procedure, map_semantic_type};

use std::path::Path;
use newcp_parser::read_module_ast;

/// Parse, analyze, and lower a source file to an `IrModule`.
pub fn lower_from_path(path: &Path) -> Result<IrModule, String> {
    let (sema_module, ast) = parse_and_analyze(path)?;
    Ok(lower_module(&sema_module, &ast))
}

fn parse_and_analyze(path: &Path) -> Result<(newcp_sema::SemanticModule, newcp_parser::ModuleAst), String> {
    let ast = read_module_ast(path).map_err(|e| e.to_string())?;
    let sema = newcp_sema::analyze_module_ast(&ast);
    Ok((sema, ast))
}

/// Produce a CFG dump (block graph with terminators) for each procedure in the
/// module at `path`.  Runs sema first to obtain the typed module, then lowers
/// to IR and renders the CFG skeleton.
pub fn dump_cfg(path: &Path) -> String {
    match parse_and_analyze(path) {
        Ok((sema_module, ast)) => {
            let ir_module = lower_module(&sema_module, &ast);
            let mut lines = vec![
                "newcp-ir CFG dump".to_string(),
                format!("input: {}", path.display()),
                format!("module: {}", ir_module.name),
            ];
            if ir_module.procedures.is_empty() {
                lines.push("procedures: <none>".to_string());
            } else {
                for proc in &ir_module.procedures {
                    lines.push(String::new());
                    lines.push(render_cfg(proc));
                }
            }
            lines.join("\n")
        }
        Err(err) => format!(
            "newcp-ir CFG error\ninput: {}\nerror: {}",
            path.display(),
            err
        ),
    }
}

/// Produce a full IR dump (instructions + terminators) for each procedure.
pub fn dump_ir(path: &Path) -> String {
    match parse_and_analyze(path) {
        Ok((sema_module, ast)) => {
            let ir_module = lower_module(&sema_module, &ast);
            let mut lines = vec![
                "newcp-ir module dump".to_string(),
                format!("input: {}", path.display()),
            ];
            lines.push(ir_module.render());
            lines.join("\n")
        }
        Err(err) => format!(
            "newcp-ir error\ninput: {}\nerror: {}",
            path.display(),
            err
        ),
    }
}

/// Render only the block headers + terminators (no instructions) -- the CFG view.
fn render_cfg(proc: &IrProcedure) -> String {
    let export_mark = if proc.exported { "*" } else { "" };
    let params = proc
        .params
        .iter()
        .map(|(name, ty)| format!("{name}: {}", ty.render()))
        .collect::<Vec<_>>()
        .join(", ");
    let ret = if proc.ret_ty == IrType::Void {
        String::new()
    } else {
        format!(" -> {}", proc.ret_ty.render())
    };

    let mut lines = vec![format!("procedure {export_mark}{} ({params}){ret} {{", proc.name)];

    // Sort blocks by RPO if available.
    let mut ids: Vec<BlockId> = proc.blocks.iter().map(|b| b.id).collect();
    ids.sort_by_key(|id| {
        let b = proc.block(*id);
        b.rpo_index
            .map(|r| r as i64)
            .unwrap_or(b.construction_index as i64 + 100_000)
    });

    for id in ids {
        let b = proc.block(id);
        let succs = proc
            .successors(id)
            .iter()
            .map(|s| s.render())
            .collect::<Vec<_>>()
            .join(", ");
        let succs_str = if succs.is_empty() {
            String::new()
        } else {
            format!("  ->  [{succs}]")
        };
        lines.push(format!(
            "  {}{}",
            b.render_header(),
            succs_str
        ));
        // Terminator only (not instructions).
        lines.push(format!("    {}", render_terminator_brief(&b.terminator)));
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn render_terminator_brief(term: &Terminator) -> String {
    match term {
        Terminator::Br { target } => format!("br {}", target.render()),
        Terminator::CondBr { cond, true_target, false_target } => {
            format!("condbr {}, {}, {}", cond.render(), true_target.render(), false_target.render())
        }
        Terminator::Ret { value } => format!("ret {}", value.render()),
        Terminator::RetVoid => "retvoid".to_string(),
        Terminator::Trap { kind } => format!("trap {}", kind.render()),
        Terminator::TypeTest { value, ty, true_target, false_target } => {
            format!(
                "typetest {} is {}, {}, {}",
                value.render(),
                ty.render(),
                true_target.render(),
                false_target.render()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use newcp_sema::{BuiltinType, SemanticType};

    #[test]
    fn cfg_dump_lists_procedure_blocks() {
        let temp = std::env::temp_dir().join("newcp-ir-test.cp");
        std::fs::write(&temp, "MODULE Demo;\nPROCEDURE Run*;\nBEGIN\nEND Run;\nEND Demo.")
            .expect("write test module");

        let dump = dump_cfg(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("Run"), "expected 'Run' in:\n{dump}");
        // The real IR will have at least entry and exit blocks.
        assert!(dump.contains("bb0"), "expected bb0 in:\n{dump}");
        assert!(dump.contains("bb1"), "expected bb1 in:\n{dump}");
    }

    #[test]
    fn ir_dump_lowers_system_intrinsics() {
        let temp = std::env::temp_dir().join("newcp-ir-system.cp");
        std::fs::write(
            &temp,
            concat!(
                "MODULE Demo;\n",
                "IMPORT SYSTEM;\n",
                "TYPE Raw = RECORD [untagged] value: INTEGER END;\n",
                "TYPE RawPtr = POINTER [untagged] TO Raw;\n",
                "VAR addr: INTEGER; p: RawPtr;\n",
                "PROCEDURE Run*;\n",
                "BEGIN\n",
                "  addr := SYSTEM.ADR(addr);\n",
                "  addr := SYSTEM.LSH(addr, 1);\n",
                "  SYSTEM.PUT(addr, 1);\n",
                "  SYSTEM.NEW(p, 64)\n",
                "END Run;\n",
                "END Demo."
            ),
        )
        .expect("write test module");

        let dump = dump_ir(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("addrof"), "expected ADR lowering in:\n{dump}");
        assert!(dump.contains("lsh"), "expected LSH lowering in:\n{dump}");
        assert!(dump.contains("store_raw"), "expected PUT lowering in:\n{dump}");
        assert!(dump.contains("sysnew"), "expected SYSTEM.NEW lowering in:\n{dump}");
    }

    #[test]
    fn map_semantic_type_lowers_intshort_to_i32() {
        assert_eq!(
            map_semantic_type(&SemanticType::Builtin(BuiltinType::IntShort)),
            IrType::I32
        );
    }
}
