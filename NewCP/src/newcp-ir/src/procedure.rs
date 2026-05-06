use std::collections::{HashMap, HashSet};

use crate::{
    ir::{BasicBlock, BlockId, Instr, TempId, Terminator},
    types::IrType,
};

/// A symbol (global variable or constant) at module scope.
#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub ty: IrType,
    pub exported: bool,
    /// True when the global is a read-only constant.
    pub is_const: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringDiagnostic {
    pub message: String,
}

impl LoweringDiagnostic {
    pub fn render(&self) -> String {
        self.message.clone()
    }
}

/// A procedure in IR form: a collection of basic blocks with one entry.
#[derive(Debug, Clone)]
pub struct IrProcedure {
    pub name: String,
    pub exported: bool,
    pub params: Vec<(String, IrType)>,
    pub ret_ty: IrType,
    /// All blocks, in construction order.
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    /// The single function-exit block (all logical RETURNs branch here).
    pub exit: BlockId,
    /// Next TempId counter, used during construction.
    next_temp: u32,
    pub diagnostics: Vec<LoweringDiagnostic>,
}

impl IrProcedure {
    pub fn new(
        name: String,
        exported: bool,
        params: Vec<(String, IrType)>,
        ret_ty: IrType,
    ) -> Self {
        Self {
            name,
            exported,
            params,
            ret_ty,
            blocks: Vec::new(),
            entry: BlockId(0),
            exit: BlockId(0),
            next_temp: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Allocate a fresh TempId.
    pub fn fresh_temp(&mut self) -> TempId {
        let id = TempId(self.next_temp);
        self.next_temp += 1;
        id
    }

    /// Allocate a fresh BlockId and register an (incomplete) block.
    /// The caller must set the terminator before the procedure is finished.
    pub fn alloc_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        let c = id.0;
        self.blocks.push(BasicBlock {
            id,
            construction_index: c,
            rpo_index: None,
            instrs: Vec::new(),
            terminator: Terminator::RetVoid, // placeholder — must be overwritten
        });
        id
    }

    /// Get a mutable reference to a block by ID.
    pub fn block_mut(&mut self, id: BlockId) -> &mut BasicBlock {
        &mut self.blocks[id.0 as usize]
    }

    /// Get a shared reference to a block by ID.
    pub fn block(&self, id: BlockId) -> &BasicBlock {
        &self.blocks[id.0 as usize]
    }

    /// Push an instruction onto a block.
    pub fn push_instr(&mut self, block: BlockId, instr: Instr) {
        self.blocks[block.0 as usize].instrs.push(instr);
    }

    /// Set the terminator of a block.
    pub fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        self.blocks[block.0 as usize].terminator = term;
    }

    /// Compute and store RPO indices for all reachable blocks.
    pub fn compute_rpo(&mut self) {
        // Clear existing indices.
        for b in &mut self.blocks {
            b.rpo_index = None;
        }

        // Post-order DFS from entry.
        let mut post_order: Vec<BlockId> = Vec::new();
        let mut visited: HashSet<u32> = HashSet::new();
        self.dfs_post(self.entry, &mut visited, &mut post_order);

        // RPO = reverse of post-order.
        let total = post_order.len() as u32;
        for (rpo_idx, block_id) in post_order.iter().rev().enumerate() {
            self.blocks[block_id.0 as usize].rpo_index = Some(rpo_idx as u32);
        }
        let _ = total; // suppress unused warning
    }

    /// Remove blocks that are unreachable from `entry`, compacting block IDs
    /// and updating terminator targets to match the new numbering.
    pub fn prune_unreachable(&mut self) {
        let mut post_order: Vec<BlockId> = Vec::new();
        let mut visited: HashSet<u32> = HashSet::new();
        self.dfs_post(self.entry, &mut visited, &mut post_order);

        if visited.len() == self.blocks.len() {
            return;
        }

        let mut id_map: HashMap<u32, BlockId> = HashMap::new();
        let mut new_blocks = Vec::with_capacity(visited.len());

        for old_block in &self.blocks {
            if !visited.contains(&old_block.id.0) {
                continue;
            }
            let new_id = BlockId(new_blocks.len() as u32);
            id_map.insert(old_block.id.0, new_id);

            let mut cloned = old_block.clone();
            cloned.id = new_id;
            cloned.construction_index = new_id.0;
            cloned.rpo_index = None;
            new_blocks.push(cloned);
        }

        for block in &mut new_blocks {
            remap_terminator_targets(&mut block.terminator, &id_map);
        }

        self.entry = id_map[&self.entry.0];
        self.exit = id_map[&self.exit.0];
        self.blocks = new_blocks;
    }

    fn dfs_post(&self, id: BlockId, visited: &mut HashSet<u32>, post_order: &mut Vec<BlockId>) {
        if !visited.insert(id.0) {
            return;
        }
        for succ in self.successors(id) {
            self.dfs_post(succ, visited, post_order);
        }
        post_order.push(id);
    }

    /// Return the successor block IDs of a block (from its terminator).
    pub fn successors(&self, id: BlockId) -> Vec<BlockId> {
        match &self.block(id).terminator {
            Terminator::Br { target } => vec![*target],
            Terminator::CondBr { true_target, false_target, .. } => {
                vec![*true_target, *false_target]
            }
            Terminator::TypeTest { true_target, false_target, .. } => {
                vec![*true_target, *false_target]
            }
            Terminator::Ret { .. } | Terminator::RetVoid | Terminator::Trap { .. } => vec![],
        }
    }

    /// Return a map from BlockId to its predecessor IDs.
    pub fn predecessors(&self) -> HashMap<u32, Vec<BlockId>> {
        let mut preds: HashMap<u32, Vec<BlockId>> = HashMap::new();
        for b in &self.blocks {
            for succ in self.successors(b.id) {
                preds.entry(succ.0).or_default().push(b.id);
            }
        }
        preds
    }

    /// Render the procedure as readable text.
    pub fn render(&self) -> String {
        let export_mark = if self.exported { "*" } else { "" };
        let params = self
            .params
            .iter()
            .map(|(name, ty)| format!("{name}: {}", ty.render()))
            .collect::<Vec<_>>()
            .join(", ");
        let ret = if self.ret_ty == IrType::Void {
            String::new()
        } else {
            format!(" -> {}", self.ret_ty.render())
        };

        let mut lines = vec![format!("proc {export_mark}{} ({params}){ret} {{", self.name)];

        if !self.diagnostics.is_empty() {
            lines.push("  diagnostics:".to_string());
            for diagnostic in &self.diagnostics {
                lines.push(format!("    error: {}", diagnostic.render()));
            }
        }

        // Sort blocks by RPO index if available, else construction order.
        let mut ids: Vec<BlockId> = self.blocks.iter().map(|b| b.id).collect();
        ids.sort_by_key(|id| {
            let b = self.block(*id);
            b.rpo_index
                .map(|r| r as i64)
                .unwrap_or(b.construction_index as i64 + 100_000)
        });

        for id in ids {
            let b = self.block(id);
            lines.push(format!("  {}", b.render_header()));
            for instr in &b.instrs {
                lines.push(format!("    {}", render_instr(instr)));
            }
            lines.push(format!("    {}", render_terminator(&b.terminator)));
        }
        lines.push("}".to_string());
        lines.join("\n")
    }
}

fn remap_terminator_targets(term: &mut Terminator, id_map: &HashMap<u32, BlockId>) {
    match term {
        Terminator::Br { target } => {
            *target = id_map[&target.0];
        }
        Terminator::CondBr {
            true_target,
            false_target,
            ..
        }
        | Terminator::TypeTest {
            true_target,
            false_target,
            ..
        } => {
            *true_target = id_map[&true_target.0];
            *false_target = id_map[&false_target.0];
        }
        Terminator::Ret { .. } | Terminator::RetVoid | Terminator::Trap { .. } => {}
    }
}

/// An entire module in IR form.
#[derive(Debug, Clone)]
pub struct IrModule {
    pub name: String,
    pub imports: Vec<String>,
    pub globals: Vec<IrGlobal>,
    pub procedures: Vec<IrProcedure>,
    /// Named record-type declarations, keyed by simple type name.
    ///
    /// Each entry is the flattened ordered field list: `(field_name, field_ir_type)`.
    /// Field index into `Instr::Gep` corresponds to the position in this list.
    pub named_types: std::collections::HashMap<String, Vec<(String, crate::types::IrType)>>,
    /// Method vtable for each record type that declares or inherits bound procedures.
    ///
    /// Key: simple type name (e.g. `"Shape"`, `"Circle"`).
    /// Value: ordered list of **LLVM function names** filling vtable slots 0, 1, …
    ///   — each entry is the concrete implementation that should occupy that slot for
    ///   objects of *this exact type* (i.e., overrides have replaced base entries).
    ///
    /// The vtable is used to:
    ///   1. Emit `@TypeName.vtable` constant arrays in LLVM IR.
    ///   2. Let the lowerer compute the slot index for a `MethodCall` instruction.
    pub type_vtables: std::collections::HashMap<String, Vec<String>>,
    /// Direct base type name for each extensible record type, if any.
    ///
    /// Key: simple type name.  Value: `Some("BaseTypeName")` or `None`.
    /// Used when emitting `TypeDesc.base` to chain the descriptor for `IS`/`WITH`.
    pub type_bases: std::collections::HashMap<String, Option<String>>,
}

impl IrModule {
    pub fn render(&self) -> String {
        let imports = if self.imports.is_empty() {
            "<none>".to_string()
        } else {
            self.imports.join(", ")
        };

        let mut lines = vec![
            format!("module {}", self.name),
            format!("imports: {imports}"),
        ];

        if self.globals.is_empty() {
            lines.push("globals: <none>".to_string());
        } else {
            lines.push("globals:".to_string());
            for g in &self.globals {
                let mark = if g.exported { "*" } else { "" };
                let kind = if g.is_const { "const" } else { "var" };
                lines.push(format!("  {kind} {mark}{} : {}", g.name, g.ty.render()));
            }
        }

        if self.procedures.is_empty() {
            lines.push("procedures: <none>".to_string());
        } else {
            for proc in &self.procedures {
                lines.push(String::new());
                lines.push(proc.render());
            }
        }

        lines.join("\n")
    }

    pub fn lowering_diagnostics(&self) -> Vec<(String, String)> {
        self.procedures
            .iter()
            .flat_map(|proc| {
                proc.diagnostics
                    .iter()
                    .map(|diagnostic| (proc.name.clone(), diagnostic.render()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn has_lowering_diagnostics(&self) -> bool {
        self.procedures.iter().any(|proc| !proc.diagnostics.is_empty())
    }
}

// ── Instruction rendering ────────────────────────────────────────────────────

fn render_instr(instr: &Instr) -> String {
    use crate::ir::Instr::*;
    match instr {
        BinOp { dst, op, left, right, ty } => {
            format!("{} : {} = {} {}, {}", dst.render(), ty.render(), op.render(), left.render(), right.render())
        }
        UnOp { dst, op, operand, ty } => {
            format!("{} : {} = {} {}", dst.render(), ty.render(), op.render(), operand.render())
        }
        Load { dst, addr, ty } => {
            format!("{} : {} = load {}", dst.render(), ty.render(), addr.render())
        }
        LoadRaw { dst, addr, ty } => {
            format!("{} : {} = load_raw {}", dst.render(), ty.render(), addr.render())
        }
        Store { addr, value } => {
            format!("store {}, {}", addr.render(), value.render())
        }
        StoreRaw { addr, value } => {
            format!("store_raw {}, {}", addr.render(), value.render())
        }
        Call { dst, callee, args, ret_ty } => {
            let args_str = args.iter().map(|a| a.render()).collect::<Vec<_>>().join(", ");
            let dst_str = dst.map(|d| format!("{} : {} = ", d.render(), ret_ty.render())).unwrap_or_default();
            format!("{dst_str}call {}({})", callee.render(), args_str)
        }
        MethodCall { dst, descriptor, slot, args, ret_ty } => {
            let args_str = args.iter().map(|a| a.render()).collect::<Vec<_>>().join(", ");
            let dst_str = dst.map(|d| format!("{} : {} = ", d.render(), ret_ty.render())).unwrap_or_default();
            format!("{dst_str}methodcall {}[{slot}]({})", descriptor.render(), args_str)
        }
        AddrOf { dst, sym } => {
            format!("{} = addrof {}", dst.render(), sym.render())
        }
        BitCast { dst, value, ty } => {
            format!("{} : {} = bitcast {}", dst.render(), ty.render(), value.render())
        }
        Cast { dst, value, to_ty } => {
            format!("{} : {} = cast {}", dst.render(), to_ty.render(), value.render())
        }
        Lsh { dst, value, shift, ty } => {
            format!("{} : {} = lsh {}, {}", dst.render(), ty.render(), value.render(), shift.render())
        }
        Ash { dst, value, shift, ty } => {
            format!("{} : {} = ash {}, {}", dst.render(), ty.render(), value.render(), shift.render())
        }
        Rot { dst, value, shift, ty } => {
            format!("{} : {} = rot {}, {}", dst.render(), ty.render(), value.render(), shift.render())
        }
        Entier { dst, value } => {
            format!("{} : i64 = entier {}", dst.render(), value.render())
        }
        MemCopy { dst, src, len } => {
            format!("memcopy {}, {}, {}", dst.render(), src.render(), len.render())
        }
        TypTag { dst, value } => {
            format!("{} : i64 = typ {}", dst.render(), value.render())
        }
        SysNew { dst, size } => {
            format!("{} : ptr<u8> = sysnew {}", dst.render(), size.render())
        }
        TypeCheck { dst, value, ty } => {
            format!("{} : bool = typecheck {} is {}", dst.render(), value.render(), ty.render())
        }
        Gep { dst, base, field_index, result_ty } => {
            format!("{} : ptr<{}> = gep {}, {}", dst.render(), result_ty.render(), base.render(), field_index)
        }
        IndexGep { dst, base, index, element_ty } => {
            format!("{} : ref<{}> = indexgep {}[{}]", dst.render(), element_ty.render(), base.render(), index.render())
        }
        New { dst, record_ty } => {
            format!("{} : ptr = new {}", dst.render(), record_ty.render())
        }
        StoreResult { value } => {
            format!("store result, {}", value.render())
        }
    }
}

fn render_terminator(term: &Terminator) -> String {
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
