/// Lowering: Component Pascal AST (via SemanticModule + ModuleAst) -> IrModule/IrProcedure.
///
/// Design notes:
/// - The CFG *is* the IR; no separate TAC pass.
/// - Every logical RETURN compiles to StoreResult (if non-void) + Br(function_exit).
/// - The function_exit block emits the physical Ret (or RetVoid).
/// - EXIT inside a LOOP emits Br(loop_exit_target).
/// - WITH arms with a None guard are the ELSE arm.
/// - After all blocks are built, RPO is computed and stored on each block.
use newcp_parser::{
    BinaryOp, CaseArm, CaseLabel, Declaration, Designator, Expr, Guard, IfBranch, Literal,
    ModuleAst, ParamMode, ProcedureDecl, QualIdent, Selector, SetElement, Statement, UnaryOp,
    WithArm,
};
use newcp_sema::{BuiltinType, SemanticModule, SemanticProcedure, SemanticSymbol, SemanticType};

use crate::{
    ir::{BinOp, BlockId, Instr, TempId, Terminator, TrapKind, UnOp},
    ir::IrValue,
    procedure::{IrGlobal, IrModule, IrProcedure},
    types::IrType,
};

// == Type mapping ==

pub fn map_semantic_type(ty: &SemanticType) -> IrType {
    match ty {
        SemanticType::Builtin(bt) => map_builtin(*bt),
        SemanticType::Nil => IrType::Ptr(Box::new(IrType::Opaque("nil".to_string()))),
        SemanticType::Named { module, name, .. } => {
            let full = match module {
                Some(m) => format!("{m}.{name}"),
                None => name.clone(),
            };
            IrType::Named(full)
        }
        SemanticType::Array { element_type, .. } => {
            IrType::Ptr(Box::new(map_semantic_type(element_type)))
        }
        SemanticType::Record { .. } => IrType::Opaque("anon-record".to_string()),
        SemanticType::Pointer { target } => IrType::Ptr(Box::new(map_semantic_type(target))),
        SemanticType::Procedure(_) => IrType::Opaque("proc-type".to_string()),
        SemanticType::BuiltinProc(_) => IrType::Opaque("builtin-proc".to_string()),
    }
}

fn map_builtin(bt: BuiltinType) -> IrType {
    match bt {
        BuiltinType::Boolean => IrType::Bool,
        BuiltinType::Byte => IrType::U8,
        BuiltinType::Char => IrType::Char,
        BuiltinType::ShortChar => IrType::ShortChar,
        BuiltinType::Integer => IrType::I32,
        BuiltinType::LongInt => IrType::I64,
        BuiltinType::ShortInt => IrType::I16,
        BuiltinType::Real => IrType::F64,
        BuiltinType::ShortReal => IrType::F32,
        BuiltinType::Set => IrType::Set(32),
        BuiltinType::String | BuiltinType::ShortString => {
            IrType::Ptr(Box::new(IrType::ShortChar))
        }
        BuiltinType::AnyPtr => IrType::Ptr(Box::new(IrType::Opaque("anyptr".to_string()))),
        BuiltinType::AnyRec => IrType::Opaque("anyrec".to_string()),
    }
}

// == Lowering context ==

struct LowerCtx<'m> {
    proc: IrProcedure,
    current: BlockId,
    loop_stack: Vec<(BlockId, BlockId)>,
    function_exit: BlockId,
    result_slot: Option<IrValue>,
    _module_symbols: &'m [SemanticSymbol],
}

impl<'m> LowerCtx<'m> {
    fn new(
        proc_ir: IrProcedure,
        entry: BlockId,
        function_exit: BlockId,
        result_slot: Option<IrValue>,
        module_symbols: &'m [SemanticSymbol],
    ) -> Self {
        Self {
            proc: proc_ir,
            current: entry,
            loop_stack: Vec::new(),
            function_exit,
            result_slot,
            _module_symbols: module_symbols,
        }
    }

    fn fresh_temp(&mut self) -> TempId {
        self.proc.fresh_temp()
    }

    fn alloc_block(&mut self) -> BlockId {
        self.proc.alloc_block()
    }

    fn push(&mut self, instr: Instr) {
        self.proc.push_instr(self.current, instr);
    }

    fn set_term(&mut self, term: Terminator) {
        self.proc.set_terminator(self.current, term);
    }

    fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }
}

// == Expression lowering ==

impl<'m> LowerCtx<'m> {
    fn lower_expr(&mut self, expr: &Expr) -> IrValue {
        match expr {
            Expr::Literal { value, .. } => self.lower_literal(value),
            Expr::Nil { .. } => {
                IrValue::Null(IrType::Ptr(Box::new(IrType::Opaque("nil".to_string()))))
            }
            Expr::Designator(des) => self.lower_designator(des),
            Expr::Unary { op, expr, .. } => self.lower_unary(*op, expr),
            Expr::Binary { left, op, right, .. } => self.lower_binary(left, *op, right),
            Expr::Set { .. } => {
                let t = self.fresh_temp();
                self.push(Instr::BitCast {
                    dst: t,
                    value: IrValue::ConstInt(0, IrType::Set(32)),
                    ty: IrType::Set(32),
                });
                IrValue::Temp(t, IrType::Set(32))
            }
        }
    }

    fn lower_literal(&self, lit: &Literal) -> IrValue {
        match lit {
            Literal::Integer(s) => {
                let v: i128 = s.parse().unwrap_or(0);
                IrValue::ConstInt(v, IrType::I32)
            }
            Literal::Real(s) => {
                let v: f64 = s.parse().unwrap_or(0.0);
                IrValue::ConstReal(v, IrType::F64)
            }
            Literal::Character(s) => {
                let inner = s.trim_matches('"').trim_matches('\'');
                let c = inner.chars().next().unwrap_or('\0');
                IrValue::ConstChar(c)
            }
            Literal::String(s) => {
                let inner = s.trim_matches('"').trim_matches('\'');
                if inner.chars().count() == 1 {
                    IrValue::ConstChar(inner.chars().next().unwrap())
                } else {
                    IrValue::ConstStr(inner.to_string())
                }
            }
        }
    }

    fn lower_designator(&mut self, des: &Designator) -> IrValue {
        let (module_opt, base_name) = match &des.base {
            QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
            QualIdent { name, .. } => (None, name.clone()),
        };

        let base_ty = IrType::Opaque("unresolved".to_string());
        let mut val: IrValue = match module_opt {
            Some(m) => {
                let t = self.fresh_temp();
                self.push(Instr::Load {
                    dst: t,
                    addr: IrValue::ImportRef(
                        m.clone(),
                        base_name.clone(),
                        IrType::Ref(Box::new(base_ty.clone())),
                    ),
                    ty: base_ty.clone(),
                });
                IrValue::Temp(t, base_ty.clone())
            }
            None => {
                let t = self.fresh_temp();
                self.push(Instr::Load {
                    dst: t,
                    addr: IrValue::GlobalRef(
                        base_name.clone(),
                        IrType::Ref(Box::new(base_ty.clone())),
                    ),
                    ty: base_ty.clone(),
                });
                IrValue::Temp(t, base_ty.clone())
            }
        };

        for sel in &des.selectors {
            val = self.lower_selector(val, sel);
        }
        val
    }

    fn lower_selector(&mut self, base: IrValue, sel: &Selector) -> IrValue {
        match sel {
            Selector::Field(fname) => {
                let t = self.fresh_temp();
                let ty = IrType::Opaque(format!("field:{fname}"));
                self.push(Instr::Load {
                    dst: t,
                    addr: IrValue::GlobalRef(
                        format!("field:{fname}"),
                        IrType::Ref(Box::new(ty.clone())),
                    ),
                    ty: ty.clone(),
                });
                let _ = base;
                IrValue::Temp(t, ty)
            }
            Selector::Index(_) | Selector::StringDereference => {
                let t = self.fresh_temp();
                let ty = IrType::Opaque("array-element".to_string());
                self.push(Instr::Load { dst: t, addr: base.clone(), ty: ty.clone() });
                IrValue::Temp(t, ty)
            }
            Selector::Dereference => {
                let t = self.fresh_temp();
                let ty = IrType::Opaque("deref".to_string());
                self.push(Instr::Load { dst: t, addr: base.clone(), ty: ty.clone() });
                IrValue::Temp(t, ty)
            }
            Selector::TypeGuard(ty_ident) => {
                let (module_opt, type_name) = match ty_ident {
                    QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
                    QualIdent { name, .. } => (None, name.clone()),
                };
                let ir_ty = match module_opt {
                    Some(m) => IrType::Named(format!("{m}.{type_name}")),
                    None => IrType::Named(type_name),
                };
                let t = self.fresh_temp();
                self.push(Instr::BitCast { dst: t, value: base, ty: ir_ty.clone() });
                IrValue::Temp(t, ir_ty)
            }
            Selector::Call(args) => {
                let args_lowered: Vec<IrValue> = args.iter().map(|a| self.lower_expr(a)).collect();
                let t = self.fresh_temp();
                let ret_ty = IrType::Opaque("call-result".to_string());
                self.push(Instr::Call {
                    dst: Some(t),
                    callee: base,
                    args: args_lowered,
                    ret_ty: ret_ty.clone(),
                });
                IrValue::Temp(t, ret_ty)
            }
            Selector::AmbiguousParen(qual) => {
                // Resolved at parse time to either a call or type guard;
                // here treat as a zero-arg call.
                let t = self.fresh_temp();
                let ret_ty = IrType::Opaque("call-result".to_string());
                self.push(Instr::Call {
                    dst: Some(t),
                    callee: base,
                    args: vec![],
                    ret_ty: ret_ty.clone(),
                });
                let _ = qual;
                IrValue::Temp(t, ret_ty)
            }
        }
    }

    fn lower_unary(&mut self, op: UnaryOp, expr: &Expr) -> IrValue {
        let operand = self.lower_expr(expr);
        let ty = operand.ty();
        let ir_op = match op {
            UnaryOp::Minus => UnOp::Neg,
            UnaryOp::Not => UnOp::Not,
            UnaryOp::Plus => return operand,
        };
        let t = self.fresh_temp();
        self.push(Instr::UnOp { dst: t, op: ir_op, operand, ty: ty.clone() });
        IrValue::Temp(t, ty)
    }

    fn lower_binary(&mut self, left: &Expr, op: BinaryOp, right: &Expr) -> IrValue {
        let lv = self.lower_expr(left);
        let rv = self.lower_expr(right);
        let ty = lv.ty();
        let result_ty = match op {
            BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::Less | BinaryOp::LessEqual
            | BinaryOp::Greater | BinaryOp::GreaterEqual | BinaryOp::In | BinaryOp::Is
            | BinaryOp::And | BinaryOp::Or => IrType::Bool,
            _ => ty.clone(),
        };

        if op == BinaryOp::Is {
            let ir_ty = match right {
                Expr::Designator(des) => {
                    let (m_opt, name) = match &des.base {
                        QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
                        QualIdent { name, .. } => (None, name.clone()),
                    };
                    match m_opt {
                        Some(m) => IrType::Named(format!("{m}.{name}")),
                        None => IrType::Named(name),
                    }
                }
                _ => IrType::Opaque("is-check".to_string()),
            };
            let t = self.fresh_temp();
            self.push(Instr::TypeCheck { dst: t, value: lv, ty: ir_ty });
            return IrValue::Temp(t, IrType::Bool);
        }

        let ir_op = match op {
            BinaryOp::Add => BinOp::Add,
            BinaryOp::Subtract => BinOp::Sub,
            BinaryOp::Multiply => BinOp::Mul,
            BinaryOp::Divide | BinaryOp::Div => BinOp::Div,
            BinaryOp::Mod => BinOp::Mod,
            BinaryOp::Equal => BinOp::Eq,
            BinaryOp::NotEqual => BinOp::Ne,
            BinaryOp::Less => BinOp::Lt,
            BinaryOp::LessEqual => BinOp::Le,
            BinaryOp::Greater => BinOp::Gt,
            BinaryOp::GreaterEqual => BinOp::Ge,
            BinaryOp::And => BinOp::And,
            BinaryOp::Or => BinOp::Or,
            BinaryOp::In => BinOp::In,
            BinaryOp::Is => unreachable!(),
        };
        let t = self.fresh_temp();
        self.push(Instr::BinOp { dst: t, op: ir_op, left: lv, right: rv, ty: result_ty.clone() });
        IrValue::Temp(t, result_ty)
    }
}

// == Statement lowering ==

impl<'m> LowerCtx<'m> {
    fn lower_statements(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            self.lower_statement(stmt);
        }
    }

    fn lower_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Empty { .. } => {}

            Statement::Assignment { target, value, .. } => {
                let rhs = self.lower_expr(value);
                let addr = self.designator_addr(target);
                self.push(Instr::Store { addr, value: rhs });
            }

            Statement::ProcedureCall { designator, .. } => {
                let _ = self.lower_designator(designator);
            }

            Statement::If { branches, else_branch, .. } => {
                self.lower_if(branches, else_branch.as_deref());
            }

            Statement::While { condition, body, .. } => {
                self.lower_while(condition, body);
            }

            Statement::Repeat { body, until, .. } => {
                self.lower_repeat(body, until);
            }

            Statement::For { variable, start, end, step, body, .. } => {
                self.lower_for(variable, start, end, step.as_ref(), body);
            }

            Statement::Loop { body, .. } => {
                self.lower_loop(body);
            }

            Statement::Exit { .. } => {
                self.lower_exit();
            }

            Statement::Return { expr, .. } => {
                self.lower_return(expr.as_ref());
            }

            Statement::Case { expr, arms, else_branch, .. } => {
                self.lower_case(expr, arms, else_branch.as_deref());
            }

            Statement::With { arms, else_branch, .. } => {
                self.lower_with(arms, else_branch.as_deref());
            }
        }
    }

    fn designator_addr(&mut self, des: &Designator) -> IrValue {
        let (module_opt, base_name) = match &des.base {
            QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
            QualIdent { name, .. } => (None, name.clone()),
        };
        match module_opt {
            Some(m) => IrValue::ImportRef(
                m,
                base_name,
                IrType::Ref(Box::new(IrType::Opaque("addr".to_string()))),
            ),
            None => IrValue::GlobalRef(
                base_name,
                IrType::Ref(Box::new(IrType::Opaque("addr".to_string()))),
            ),
        }
    }

    // -- IF / ELSIF / ELSE --

    fn lower_if(&mut self, branches: &[IfBranch], else_branch: Option<&[Statement]>) {
        let merge_block = self.alloc_block();

        let branch_blocks: Vec<(BlockId, Option<BlockId>)> = branches
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let body = self.alloc_block();
                let next_cond = if i + 1 < branches.len() {
                    Some(self.alloc_block())
                } else {
                    None
                };
                (body, next_cond)
            })
            .collect();

        let else_block = if else_branch.is_some() {
            Some(self.alloc_block())
        } else {
            None
        };

        for (idx, branch) in branches.iter().enumerate() {
            let (body_block, next_cond_block) = branch_blocks[idx];
            let false_target = next_cond_block.or(else_block).unwrap_or(merge_block);

            let cond = self.lower_expr(&branch.condition);
            self.set_term(Terminator::CondBr {
                cond,
                true_target: body_block,
                false_target,
            });

            self.switch_to(body_block);
            self.lower_statements(&branch.body);
            self.set_term(Terminator::Br { target: merge_block });

            if let Some(nc) = next_cond_block {
                self.switch_to(nc);
            }
        }

        if let (Some(eb), Some(else_stmts)) = (else_block, else_branch) {
            self.switch_to(eb);
            self.lower_statements(else_stmts);
            self.set_term(Terminator::Br { target: merge_block });
        }

        self.switch_to(merge_block);
    }

    // -- WHILE --

    fn lower_while(&mut self, condition: &Expr, body: &[Statement]) {
        let cond_block = self.alloc_block();
        let body_block = self.alloc_block();
        let exit_block = self.alloc_block();

        self.set_term(Terminator::Br { target: cond_block });

        self.switch_to(cond_block);
        let cond_val = self.lower_expr(condition);
        self.set_term(Terminator::CondBr {
            cond: cond_val,
            true_target: body_block,
            false_target: exit_block,
        });

        self.switch_to(body_block);
        self.lower_statements(body);
        self.set_term(Terminator::Br { target: cond_block });

        self.switch_to(exit_block);
    }

    // -- REPEAT --

    fn lower_repeat(&mut self, body: &[Statement], until: &Expr) {
        let body_block = self.alloc_block();
        let exit_block = self.alloc_block();

        self.set_term(Terminator::Br { target: body_block });
        self.switch_to(body_block);
        self.lower_statements(body);

        let cond_val = self.lower_expr(until);
        self.set_term(Terminator::CondBr {
            cond: cond_val,
            true_target: exit_block,
            false_target: body_block,
        });

        self.switch_to(exit_block);
    }

    // -- FOR --

    fn lower_for(
        &mut self,
        variable: &str,
        start: &Expr,
        end: &Expr,
        step: Option<&Expr>,
        body: &[Statement],
    ) {
        let var_addr = IrValue::GlobalRef(
            variable.to_string(),
            IrType::Ref(Box::new(IrType::I32)),
        );

        let start_val = self.lower_expr(start);
        self.push(Instr::Store { addr: var_addr.clone(), value: start_val });

        let cond_block = self.alloc_block();
        let body_block = self.alloc_block();
        let incr_block = self.alloc_block();
        let exit_block = self.alloc_block();

        self.set_term(Terminator::Br { target: cond_block });

        self.switch_to(cond_block);
        let var_t = self.fresh_temp();
        self.push(Instr::Load { dst: var_t, addr: var_addr.clone(), ty: IrType::I32 });
        let end_val = self.lower_expr(end);
        let cmp_t = self.fresh_temp();
        self.push(Instr::BinOp {
            dst: cmp_t,
            op: BinOp::Le,
            left: IrValue::Temp(var_t, IrType::I32),
            right: end_val,
            ty: IrType::Bool,
        });
        self.set_term(Terminator::CondBr {
            cond: IrValue::Temp(cmp_t, IrType::Bool),
            true_target: body_block,
            false_target: exit_block,
        });

        self.switch_to(body_block);
        self.lower_statements(body);
        self.set_term(Terminator::Br { target: incr_block });

        self.switch_to(incr_block);
        let var_t2 = self.fresh_temp();
        self.push(Instr::Load { dst: var_t2, addr: var_addr.clone(), ty: IrType::I32 });
        let step_val = step
            .map(|s| self.lower_expr(s))
            .unwrap_or(IrValue::ConstInt(1, IrType::I32));
        let new_t = self.fresh_temp();
        self.push(Instr::BinOp {
            dst: new_t,
            op: BinOp::Add,
            left: IrValue::Temp(var_t2, IrType::I32),
            right: step_val,
            ty: IrType::I32,
        });
        self.push(Instr::Store {
            addr: var_addr,
            value: IrValue::Temp(new_t, IrType::I32),
        });
        self.set_term(Terminator::Br { target: cond_block });

        self.switch_to(exit_block);
    }

    // -- LOOP / EXIT --

    fn lower_loop(&mut self, body: &[Statement]) {
        let loop_body = self.alloc_block();
        let loop_exit = self.alloc_block();

        self.set_term(Terminator::Br { target: loop_body });
        self.switch_to(loop_body);

        self.loop_stack.push((loop_body, loop_exit));
        self.lower_statements(body);
        self.loop_stack.pop();

        self.set_term(Terminator::Br { target: loop_body });

        self.switch_to(loop_exit);
    }

    fn lower_exit(&mut self) {
        let exit_target = self
            .loop_stack
            .last()
            .map(|(_, exit)| *exit)
            .unwrap_or(self.function_exit);

        self.set_term(Terminator::Br { target: exit_target });

        let dead = self.alloc_block();
        self.switch_to(dead);
    }

    // -- RETURN --

    fn lower_return(&mut self, expr: Option<&Expr>) {
        if self.result_slot.is_some() {
            if let Some(ret_expr) = expr {
                let val = self.lower_expr(ret_expr);
                self.push(Instr::StoreResult { value: val });
            }
        }
        self.set_term(Terminator::Br { target: self.function_exit });

        let dead = self.alloc_block();
        self.switch_to(dead);
    }

    // -- CASE --

    fn lower_case(
        &mut self,
        subject: &Expr,
        arms: &[CaseArm],
        else_branch: Option<&[Statement]>,
    ) {
        let subject_val = self.lower_expr(subject);
        let merge_block = self.alloc_block();

        let arm_body_blocks: Vec<BlockId> = arms.iter().map(|_| self.alloc_block()).collect();
        let else_block = if else_branch.is_some() {
            Some(self.alloc_block())
        } else {
            None
        };
        let trap_block = self.alloc_block();
        let final_fallthrough = else_block.unwrap_or(trap_block);

        for (arm_idx, arm) in arms.iter().enumerate() {
            let body_block = arm_body_blocks[arm_idx];
            let next_test = if arm_idx + 1 < arms.len() {
                arm_body_blocks[arm_idx + 1]  // we will set current to this below
            } else {
                final_fallthrough
            };

            self.lower_case_labels(&subject_val.clone(), &arm.labels, body_block, next_test);

            self.switch_to(body_block);
            self.lower_statements(&arm.body);
            self.set_term(Terminator::Br { target: merge_block });

            if arm_idx + 1 < arms.len() {
                // Allocate a fresh test-chain block for the next arm's labels.
                let next_chain = self.alloc_block();
                // Rewrite the last ConditionalBr false_target to point to next_chain
                // instead of next arm body -- handled in lower_case_labels already.
                self.switch_to(next_chain);
                let _ = next_test;
            }
        }

        if let (Some(eb), Some(else_stmts)) = (else_block, else_branch) {
            self.switch_to(eb);
            self.lower_statements(else_stmts);
            self.set_term(Terminator::Br { target: merge_block });
        }

        self.switch_to(trap_block);
        self.set_term(Terminator::Trap { kind: TrapKind::CaseFallthrough });

        self.switch_to(merge_block);
    }

    /// Emit comparisons for a list of case labels.
    /// After the last label comparison, if no label matched, branch to `miss`.
    /// If any label matched, branch to `hit`.
    fn lower_case_labels(
        &mut self,
        subject: &IrValue,
        labels: &[CaseLabel],
        hit: BlockId,
        miss: BlockId,
    ) {
        if labels.is_empty() {
            self.set_term(Terminator::Br { target: miss });
            return;
        }

        // Emit a chain: for each label, test and branch to hit or next label test.
        // The last label branches to miss if it doesn't match.
        let n = labels.len();
        for (i, label) in labels.iter().enumerate() {
            let is_last = i + 1 == n;
            let next = if is_last {
                miss
            } else {
                self.alloc_block()
            };

            let cond = if let Some(end_expr) = &label.end {
                let start_val = self.lower_expr(&label.start);
                let end_val = self.lower_expr(end_expr);
                let ge_t = self.fresh_temp();
                self.push(Instr::BinOp {
                    dst: ge_t,
                    op: BinOp::Ge,
                    left: subject.clone(),
                    right: start_val,
                    ty: IrType::Bool,
                });
                let le_t = self.fresh_temp();
                self.push(Instr::BinOp {
                    dst: le_t,
                    op: BinOp::Le,
                    left: subject.clone(),
                    right: end_val,
                    ty: IrType::Bool,
                });
                let and_t = self.fresh_temp();
                self.push(Instr::BinOp {
                    dst: and_t,
                    op: BinOp::And,
                    left: IrValue::Temp(ge_t, IrType::Bool),
                    right: IrValue::Temp(le_t, IrType::Bool),
                    ty: IrType::Bool,
                });
                IrValue::Temp(and_t, IrType::Bool)
            } else {
                let start_val = self.lower_expr(&label.start);
                let eq_t = self.fresh_temp();
                self.push(Instr::BinOp {
                    dst: eq_t,
                    op: BinOp::Eq,
                    left: subject.clone(),
                    right: start_val,
                    ty: IrType::Bool,
                });
                IrValue::Temp(eq_t, IrType::Bool)
            };

            self.set_term(Terminator::CondBr {
                cond,
                true_target: hit,
                false_target: next,
            });

            if !is_last {
                self.switch_to(next);
            }
        }
    }

    // -- WITH --

    fn lower_with(&mut self, arms: &[WithArm], else_branch: Option<&[Statement]>) {
        let merge_block = self.alloc_block();

        for arm in arms {
            let body_block = self.alloc_block();
            let next_block = self.alloc_block();

            if let Some(guard) = &arm.guard {
                let var_name = guard.variable.name.clone();
                let (m_opt, ty_name) = match &guard.ty {
                    QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
                    QualIdent { name, .. } => (None, name.clone()),
                };
                let guard_ty = match m_opt {
                    Some(m) => IrType::Named(format!("{m}.{ty_name}")),
                    None => IrType::Named(ty_name),
                };
                let subject_t = self.fresh_temp();
                let subject_addr = IrValue::GlobalRef(
                    var_name,
                    IrType::Ref(Box::new(IrType::Opaque("with-subject".to_string()))),
                );
                self.push(Instr::Load {
                    dst: subject_t,
                    addr: subject_addr,
                    ty: IrType::Opaque("with-subject".to_string()),
                });
                let subject_val = IrValue::Temp(
                    subject_t,
                    IrType::Opaque("with-subject".to_string()),
                );

                self.set_term(Terminator::TypeTest {
                    value: subject_val,
                    ty: guard_ty,
                    true_target: body_block,
                    false_target: next_block,
                });
            } else {
                // ELSE arm -- always taken.
                self.set_term(Terminator::Br { target: body_block });
            }

            self.switch_to(body_block);
            self.lower_statements(&arm.body);
            self.set_term(Terminator::Br { target: merge_block });

            self.switch_to(next_block);
        }

        if let Some(else_stmts) = else_branch {
            self.lower_statements(else_stmts);
            self.set_term(Terminator::Br { target: merge_block });
        } else {
            self.set_term(Terminator::Trap { kind: TrapKind::TypeGuard });
        }

        self.switch_to(merge_block);
    }
}

// == Procedure lowering ==

pub fn lower_procedure(
    sema_proc: &SemanticProcedure,
    ast_proc: &ProcedureDecl,
    module_symbols: &[SemanticSymbol],
) -> IrProcedure {
    let params: Vec<(String, IrType)> = sema_proc
        .signature
        .parameters
        .iter()
        .flat_map(|param| {
            let base_ty = map_semantic_type(&param.ty);
            let ir_ty = match param.mode {
                Some(ParamMode::Var) | Some(ParamMode::Out) => {
                    IrType::Ref(Box::new(base_ty))
                }
                Some(ParamMode::In) => IrType::Ref(Box::new(base_ty)),
                None => base_ty,
            };
            param.names.iter().map(move |name| (name.clone(), ir_ty.clone()))
        })
        .collect();

    let ret_ty = sema_proc
        .signature
        .result_type
        .as_ref()
        .map(|t| map_semantic_type(t))
        .unwrap_or(IrType::Void);

    let mut proc = IrProcedure::new(
        sema_proc.name.clone(),
        sema_proc.exported,
        params,
        ret_ty.clone(),
    );

    let entry = proc.alloc_block();
    let function_exit = proc.alloc_block();

    proc.entry = entry;
    proc.exit = function_exit;

    let result_slot: Option<IrValue> = if ret_ty != IrType::Void {
        Some(IrValue::GlobalRef(
            "$result".to_string(),
            IrType::Ref(Box::new(ret_ty.clone())),
        ))
    } else {
        None
    };

    let mut ctx = LowerCtx::new(
        proc,
        entry,
        function_exit,
        result_slot.clone(),
        module_symbols,
    );

    ctx.switch_to(entry);

    if let Some(body) = &ast_proc.body {
        if let Some(stmts) = &body.body {
            ctx.lower_statements(stmts);
        }
    }

    ctx.set_term(Terminator::Br { target: function_exit });

    ctx.switch_to(function_exit);
    let exit_term = if ret_ty != IrType::Void {
        let t = ctx.fresh_temp();
        ctx.push(Instr::Load {
            dst: t,
            addr: result_slot.unwrap(),
            ty: ret_ty.clone(),
        });
        Terminator::Ret { value: IrValue::Temp(t, ret_ty) }
    } else {
        Terminator::RetVoid
    };
    ctx.set_term(exit_term);

    ctx.proc.compute_rpo();
    ctx.proc
}

// == Module lowering ==

pub fn lower_module(sema: &SemanticModule, ast: &ModuleAst) -> IrModule {
    use newcp_sema::SymbolKind;

    let globals: Vec<IrGlobal> = sema
        .symbols
        .iter()
        .filter(|s| !matches!(s.kind, SymbolKind::Type | SymbolKind::Procedure | SymbolKind::Import))
        .map(|s| IrGlobal {
            name: s.name.clone(),
            ty: s
                .declared_type
                .as_ref()
                .map(map_semantic_type)
                .unwrap_or(IrType::Opaque("unknown".to_string())),
            exported: s.exported,
            is_const: matches!(s.kind, SymbolKind::Constant),
        })
        .collect();

    // Collect top-level ProcedureDecls from the AST.
    let ast_procs: Vec<&ProcedureDecl> = ast
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Procedure(p) => Some(p),
            _ => None,
        })
        .collect();

    let procedures: Vec<IrProcedure> = sema
        .procedures
        .iter()
        .filter_map(|sema_proc| {
            // Match by name.
            ast_procs
                .iter()
                .find(|p| p.heading.name.name == sema_proc.name)
                .map(|ast_proc| lower_procedure(sema_proc, ast_proc, &sema.symbols))
        })
        .collect();

    IrModule {
        name: sema.name.clone(),
        imports: sema.imports.clone(),
        globals,
        procedures,
    }
}
