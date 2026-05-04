/// Lowering: Component Pascal AST (via SemanticModule + ModuleAst) -> IrModule/IrProcedure.
///
/// Design notes:
/// - The CFG *is* the IR; no separate TAC pass.
/// - Every logical RETURN compiles to StoreResult (if non-void) + Br(function_exit).
/// - The function_exit block emits the physical Ret (or RetVoid).
/// - EXIT inside a LOOP emits Br(loop_exit_target).
/// - WITH arms with a None guard are the ELSE arm.
/// - After all blocks are built, RPO is computed and stored on each block.
use std::path::Path;

use newcp_parser::{
    read_module_ast,
    BinaryOp, CaseArm, CaseLabel, Declaration, Designator, Expr, IfBranch, Literal,
    ModuleAst, ParamMode, ProcedureDecl, QualIdent, Selector, Statement, UnaryOp,
    WithArm,
};
use newcp_sema::{analyze_module_ast, BuiltinType, ConstValue, RecordLayout as SemanticRecordLayout, SemanticModule, SemanticProcedure, SemanticSymbol, SemanticType, SymbolKind};

use crate::{
    ir::{BinOp, BlockId, Instr, TempId, Terminator, TrapKind, UnOp},
    ir::IrValue,
    procedure::{IrGlobal, IrModule, IrProcedure},
    types::{IrType, RecordLayout},
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
        SemanticType::Array { element_type, untagged, .. } => {
            if *untagged {
                IrType::UntaggedPtr(Box::new(map_semantic_type(element_type)))
            } else {
                IrType::Ptr(Box::new(map_semantic_type(element_type)))
            }
        }
        SemanticType::Record { layout, .. } => match layout {
            SemanticRecordLayout::Tagged => IrType::Opaque("anon-record".to_string()),
            _ => IrType::UntaggedRecord {
                name: "anon-record".to_string(),
                layout: map_record_layout(*layout),
            },
        },
        SemanticType::Pointer { target, untagged } => {
            if *untagged {
                IrType::UntaggedPtr(Box::new(map_semantic_type(target)))
            } else {
                IrType::Ptr(Box::new(map_semantic_type(target)))
            }
        }
        SemanticType::Procedure(_) => IrType::Opaque("proc-type".to_string()),
        SemanticType::BuiltinProc(_) => IrType::Opaque("builtin-proc".to_string()),
    }
}

fn map_record_layout(layout: SemanticRecordLayout) -> RecordLayout {
    match layout {
        SemanticRecordLayout::Tagged => RecordLayout::Tagged,
        SemanticRecordLayout::Untagged => RecordLayout::Untagged,
        SemanticRecordLayout::UntaggedNoAlign => RecordLayout::UntaggedNoAlign,
        SemanticRecordLayout::UntaggedAlign2 => RecordLayout::UntaggedAlign2,
        SemanticRecordLayout::UntaggedAlign8 => RecordLayout::UntaggedAlign8,
        SemanticRecordLayout::Union => RecordLayout::Union,
    }
}

fn map_builtin(bt: BuiltinType) -> IrType {
    match bt {
        BuiltinType::Boolean => IrType::Bool,
        BuiltinType::Byte => IrType::U8,
        BuiltinType::Char => IrType::Char,
        BuiltinType::ShortChar => IrType::ShortChar,
        BuiltinType::IntShort => IrType::I32,
        BuiltinType::Integer => IrType::I64,
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
    symbols: Vec<SemanticSymbol>,
    system_qualifiers: Vec<String>,
    module_symbols: &'m [SemanticSymbol],
}

impl<'m> LowerCtx<'m> {
    fn new(
        proc_ir: IrProcedure,
        entry: BlockId,
        function_exit: BlockId,
        result_slot: Option<IrValue>,
        symbols: Vec<SemanticSymbol>,
        system_qualifiers: Vec<String>,
        module_symbols: &'m [SemanticSymbol],
    ) -> Self {
        Self {
            proc: proc_ir,
            current: entry,
            loop_stack: Vec::new(),
            function_exit,
            result_slot,
            symbols,
            system_qualifiers,
            module_symbols,
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

    /// Look up the `SemanticType::Record` for a named type by its simple name.
    ///
    /// Searches procedure-local symbols first (for local type aliases), then
    /// falls back to module-level symbols (TYPE declarations).
    fn resolve_record_type(&self, type_name: &str) -> Option<&SemanticType> {
        let ty = self
            .symbols
            .iter()
            .rev()
            .chain(self.module_symbols.iter().rev())
            .find(|sym| sym.kind == SymbolKind::Type && sym.name == type_name)
            .and_then(|sym| sym.declared_type.as_ref())?;

        match ty {
            SemanticType::Named { module: None, name, .. } if name != type_name => {
                self.resolve_record_type(name)
            }
            _ => Some(ty),
        }
    }

    /// Given a record `SemanticType`, return the flattened list of `(name, SemanticType)` pairs
    /// for all fields (including inherited ones from the base chain, base fields first).
    fn flatten_record_fields(ty: &SemanticType) -> Vec<(String, SemanticType)> {
        let SemanticType::Record { base, fields, .. } = ty else {
            return Vec::new();
        };
        let mut result = Vec::new();
        // Recursively include base record fields first.
        if let Some(base_ty) = base {
            result.extend(Self::flatten_record_fields(base_ty));
        }
        for field in fields {
            for name in &field.names {
                result.push((name.clone(), field.ty.clone()));
            }
        }
        result
    }

    /// Resolve the pointee record type from an IrType, stripping one pointer level.
    ///
    /// For `Ptr(Named("T"))` or `Named("T")` it looks up `T` in the symbol table.
    /// Returns `None` when the base type isn't a recognized record-backed pointer.
    fn resolve_base_record_type(&self, base_ir_ty: &IrType) -> Option<&SemanticType> {
        let mut cursor = base_ir_ty;
        loop {
            match cursor {
                IrType::Ptr(inner) | IrType::UntaggedPtr(inner) | IrType::Ref(inner) => {
                    cursor = inner.as_ref();
                }
                IrType::Named(n) => break self.resolve_record_type(n.as_str()),
                _ => break None,
            }
        }
    }

    fn base_symbol_ir_type(&self, qual: &QualIdent) -> Option<IrType> {
        self.proc
            .params
            .iter()
            .find(|(name, _)| *name == qual.name)
            .map(|(_, ty)| ty.clone())
            .or_else(|| {
                self.symbols
                    .iter()
                    .rev()
                    .find(|symbol| symbol.name == qual.name)
                    .and_then(|symbol| symbol.declared_type.as_ref())
                    .map(map_semantic_type)
            })
    }
}

// == Expression lowering ==

impl<'m> LowerCtx<'m> {
    fn normalize_designator(&self, des: &Designator) -> Designator {
        let Some(module_name) = des.base.module.as_ref() else {
            return des.clone();
        };
        let local_base = QualIdent {
            span: des.base.span,
            module: None,
            name: module_name.clone(),
        };
        if self.base_symbol_ir_type(&local_base).is_none() {
            return des.clone();
        }

        let mut normalized = des.clone();
        let field_name = normalized.base.name.clone();
        normalized.base.name = module_name.clone();
        normalized.base.module = None;
        normalized.selectors.insert(0, Selector::Field(field_name));
        normalized
    }

    fn lower_expr(&mut self, expr: &Expr) -> IrValue {
        match expr {
            Expr::Literal { value, .. } => self.lower_literal(value),
            Expr::Nil { .. } => {
                IrValue::Null(IrType::Ptr(Box::new(IrType::Opaque("nil".to_string()))))
            }
            Expr::Designator(des) => self.lower_system_expr(des).unwrap_or_else(|| self.lower_designator(des)),
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
                IrValue::ConstInt(v, IrType::I64)
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
        let des = self.normalize_designator(des);
        let (module_opt, base_name) = match &des.base {
            QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
            QualIdent { name, .. } => (None, name.clone()),
        };

        let final_ty = self
            .designator_ir_type(&des)
            .unwrap_or_else(|| IrType::Opaque("unresolved".to_string()));
        if des.selectors.is_empty() {
            if let Some(value) = self.lower_const_designator(module_opt.as_deref(), &base_name, &final_ty) {
                return value;
            }
        }

        if self.is_direct_callee(module_opt.as_deref(), &base_name, &des.selectors) {
            let callee = match module_opt {
                Some(m) => IrValue::ImportRef(m, base_name, final_ty.clone()),
                None => IrValue::GlobalRef(base_name, final_ty.clone()),
            };

            match des.selectors.first() {
                Some(Selector::Call(args)) => {
                    let args_lowered = self.lower_call_args(&callee, args);
                    let ret_ty = self.callee_return_type(&callee);
                    if ret_ty == IrType::Void {
                        self.push(Instr::Call {
                            dst: None,
                            callee: callee.clone(),
                            args: args_lowered,
                            ret_ty,
                        });
                        return callee;
                    }

                    let t = self.fresh_temp();
                    self.push(Instr::Call {
                        dst: Some(t),
                        callee,
                        args: args_lowered,
                        ret_ty: ret_ty.clone(),
                    });
                    return IrValue::Temp(t, ret_ty);
                }
                Some(Selector::AmbiguousParen(qual)) => {
                    let arg = Expr::Designator(Designator {
                        span: qual.span,
                        base: qual.clone(),
                        selectors: Vec::new(),
                    });
                    let args_lowered = self.lower_call_args(&callee, &[arg]);
                    let ret_ty = self.callee_return_type(&callee);
                    if ret_ty == IrType::Void {
                        self.push(Instr::Call {
                            dst: None,
                            callee: callee.clone(),
                            args: args_lowered,
                            ret_ty,
                        });
                        return callee;
                    }

                    let t = self.fresh_temp();
                    self.push(Instr::Call {
                        dst: Some(t),
                        callee,
                        args: args_lowered,
                        ret_ty: ret_ty.clone(),
                    });
                    return IrValue::Temp(t, ret_ty);
                }
                _ => return callee,
            }
        }

        let addr = self.designator_addr(&des);
        let t = self.fresh_temp();
        self.push(Instr::Load {
            dst: t,
            addr,
            ty: final_ty.clone(),
        });
        IrValue::Temp(t, final_ty)
    }

    fn lower_const_designator(
        &self,
        module_name: Option<&str>,
        base_name: &str,
        ty: &IrType,
    ) -> Option<IrValue> {
        if module_name.is_some() {
            return None;
        }

        let symbol = self
            .symbols
            .iter()
            .rev()
            .find(|symbol| symbol.name == base_name)?;
        let const_value = symbol.const_value.as_ref()?;
        Some(match const_value {
            ConstValue::Integer(value) => IrValue::ConstInt(*value, ty.clone()),
            ConstValue::Real(value) => IrValue::ConstReal(*value, ty.clone()),
            ConstValue::String(value) => IrValue::ConstStr(value.clone()),
            ConstValue::Char(value) => IrValue::ConstChar(*value),
            ConstValue::Boolean(value) => IrValue::ConstBool(*value),
        })
    }

    fn designator_addr(&mut self, des: &Designator) -> IrValue {
        let des = self.normalize_designator(des);
        let (module_opt, base_name) = match &des.base {
            QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
            QualIdent { name, .. } => (None, name.clone()),
        };
        let base_ty = self
            .base_symbol_ir_type(&des.base)
            .unwrap_or_else(|| IrType::Opaque("addr".to_string()));
        let mut addr = match &module_opt {
            Some(m) => IrValue::ImportRef(m.clone(), base_name.clone(), IrType::Ref(Box::new(base_ty))),
            None => IrValue::GlobalRef(base_name.clone(), IrType::Ref(Box::new(base_ty))),
        };

        for selector in &des.selectors {
            match selector {
                Selector::Field(fname) => {
                    let mut gep_base = addr.clone();
                    let mut base_ty = addr.ty();
                    if let IrType::Ref(inner) = &base_ty {
                        if matches!(inner.as_ref(), IrType::Ref(_) | IrType::Ptr(_) | IrType::UntaggedPtr(_)) {
                            let loaded_ty = inner.as_ref().clone();
                            let t = self.fresh_temp();
                            self.push(Instr::Load {
                                dst: t,
                                addr: addr.clone(),
                                ty: loaded_ty.clone(),
                            });
                            gep_base = IrValue::Temp(t, loaded_ty.clone());
                            base_ty = loaded_ty;
                        }
                    }

                    if let Some(record_ty) = self.resolve_base_record_type(&base_ty) {
                        let flat_fields = Self::flatten_record_fields(record_ty);
                        if let Some((idx, (_, field_sem_ty))) = flat_fields
                            .iter()
                            .enumerate()
                            .find(|(_, (name, _))| name == fname)
                        {
                            let field_ty = map_semantic_type(field_sem_ty);
                            let t = self.fresh_temp();
                            self.push(Instr::Gep {
                                dst: t,
                                base: gep_base,
                                field_index: idx as u32,
                                result_ty: field_ty.clone(),
                            });
                            addr = IrValue::Temp(t, IrType::Ref(Box::new(field_ty)));
                            continue;
                        }
                    }

                    let unresolved = IrType::Opaque(format!("field:{fname}"));
                    addr = IrValue::GlobalRef(
                        format!("field:{fname}"),
                        IrType::Ref(Box::new(unresolved)),
                    );
                }
                _ => {
                    let pointee_ty = self
                        .designator_ir_type(&des)
                        .unwrap_or_else(|| IrType::Opaque("addr".to_string()));
                    addr = match &module_opt {
                        Some(m) => IrValue::ImportRef(m.clone(), des.base.name.clone(), IrType::Ref(Box::new(pointee_ty))),
                        None => IrValue::GlobalRef(des.base.name.clone(), IrType::Ref(Box::new(pointee_ty))),
                    };
                }
            }
        }
        addr
    }

    fn is_direct_callee(
        &self,
        module_name: Option<&str>,
        base_name: &str,
        selectors: &[Selector],
    ) -> bool {
        if !matches!(selectors.first(), Some(Selector::Call(_)) | Some(Selector::AmbiguousParen(_))) {
            return false;
        }

        if module_name.is_some() {
            return true;
        }

        self.symbols
            .iter()
            .rev()
            .find(|symbol| symbol.name == base_name)
            .map(|symbol| matches!(symbol.kind, SymbolKind::Procedure))
            .unwrap_or(false)
    }

    fn callee_return_type(&self, callee: &IrValue) -> IrType {
        match self.callee_procedure_type(callee) {
            Some(proc_ty) => proc_ty
                .result_type
                .as_ref()
                .map(|ty| map_semantic_type(ty.as_ref()))
                .unwrap_or(IrType::Void),
            _ => IrType::Opaque("call-result".to_string()),
        }
    }

    fn callee_procedure_type(&self, callee: &IrValue) -> Option<newcp_sema::ProcedureType> {
        match callee {
            IrValue::GlobalRef(name, _) => self
                .symbols
                .iter()
                .rev()
                .find(|symbol| symbol.name == *name)
                .and_then(|symbol| symbol.declared_type.as_ref())
                .and_then(|ty| match ty {
                    SemanticType::Procedure(proc_ty) => Some(proc_ty.clone()),
                    _ => None,
                }),
            IrValue::ImportRef(module, name, _) => self.imported_callee_procedure_type(module, name),
            _ => None,
        }
    }

    fn imported_callee_procedure_type(
        &self,
        module: &str,
        name: &str,
    ) -> Option<newcp_sema::ProcedureType> {
        let path = Path::new("Mod").join(format!("{module}.cp"));
        let ast = read_module_ast(&path).ok()?;
        let sema = analyze_module_ast(&ast);
        sema.procedures
            .iter()
            .find(|proc| proc.name == name && proc.exported)
            .map(|proc| proc.signature.clone())
    }

    fn lower_call_args(&mut self, callee: &IrValue, args: &[Expr]) -> Vec<IrValue> {
        let expected_modes = self
            .callee_procedure_type(callee)
            .map(|proc_ty| {
                proc_ty
                    .parameters
                    .iter()
                    .flat_map(|param| std::iter::repeat_n(param.mode, param.names.len()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        args.iter()
            .enumerate()
            .map(|(index, arg)| match expected_modes.get(index).copied().flatten() {
                Some(ParamMode::Var) | Some(ParamMode::Out) => match arg {
                    Expr::Designator(des) => self.designator_addr(des),
                    _ => self.lower_expr(arg),
                },
                _ => self.lower_expr(arg),
            })
            .collect()
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

    fn lower_system_expr(&mut self, des: &Designator) -> Option<IrValue> {
        let intrinsic = self.system_intrinsic(des)?;
        let args: Vec<Expr> = match des.selectors.last()? {
            Selector::Call(args) => args.clone(),
            Selector::AmbiguousParen(qual) => vec![Expr::Designator(Designator {
                span: qual.span,
                base: qual.clone(),
                selectors: Vec::new(),
            })],
            _ => return None,
        };

        match intrinsic {
            "ADR" => {
                let Expr::Designator(target) = args.first()? else {
                    return None;
                };
                let dst = self.fresh_temp();
                let sym = self.designator_addr(target);
                self.push(Instr::AddrOf { dst, sym });
                Some(IrValue::Temp(dst, IrType::I64))
            }
            "VAL" => {
                let ty = self.ir_type_from_type_arg(args.first()?)?;
                let value = self.lower_expr(args.get(1)?);
                let dst = self.fresh_temp();
                self.push(Instr::BitCast { dst, value, ty: ty.clone() });
                Some(IrValue::Temp(dst, ty))
            }
            "LSH" => {
                let value = self.lower_expr(args.first()?);
                let shift = self.lower_expr(args.get(1)?);
                let dst = self.fresh_temp();
                let ty = value.ty();
                self.push(Instr::Lsh { dst, value, shift, ty: ty.clone() });
                Some(IrValue::Temp(dst, ty))
            }
            "ROT" => {
                let value = self.lower_expr(args.first()?);
                let shift = self.lower_expr(args.get(1)?);
                let dst = self.fresh_temp();
                let ty = value.ty();
                self.push(Instr::Rot { dst, value, shift, ty: ty.clone() });
                Some(IrValue::Temp(dst, ty))
            }
            "TYP" => {
                let value = match args.first()? {
                    Expr::Designator(des) => self.designator_addr(des),
                    expr => self.lower_expr(expr),
                };
                let dst = self.fresh_temp();
                self.push(Instr::TypTag { dst, value });
                Some(IrValue::Temp(dst, IrType::I64))
            }
            _ => None,
        }
    }

    fn lower_system_statement(&mut self, des: &Designator) -> bool {
        let Some(intrinsic) = self.system_intrinsic(des) else {
            return false;
        };
        let Some(Selector::Call(args)) = des.selectors.last() else {
            return false;
        };

        match intrinsic {
            "GET" => {
                if let (Some(addr_expr), Some(Expr::Designator(target))) = (args.first(), args.get(1)) {
                    let addr = self.lower_expr(addr_expr);
                    let ty = self.designator_ir_type(target).unwrap_or(IrType::Opaque("system-get".to_string()));
                    let tmp = self.fresh_temp();
                    self.push(Instr::LoadRaw { dst: tmp, addr, ty: ty.clone() });
                    let target_addr = self.designator_addr(target);
                    self.push(Instr::Store { addr: target_addr, value: IrValue::Temp(tmp, ty) });
                    return true;
                }
            }
            "PUT" => {
                if let (Some(addr_expr), Some(value_expr)) = (args.first(), args.get(1)) {
                    let addr = self.lower_expr(addr_expr);
                    let value = self.lower_expr(value_expr);
                    self.push(Instr::StoreRaw { addr, value });
                    return true;
                }
            }
            "MOVE" => {
                if let (Some(dst_expr), Some(src_expr), Some(len_expr)) = (args.first(), args.get(1), args.get(2)) {
                    let dst = self.lower_expr(dst_expr);
                    let src = self.lower_expr(src_expr);
                    let len = self.lower_expr(len_expr);
                    self.push(Instr::MemCopy { dst, src, len });
                    return true;
                }
            }
            "NEW" => {
                if let (Some(Expr::Designator(target)), Some(size_expr)) = (args.first(), args.get(1)) {
                    let size = self.lower_expr(size_expr);
                    let tmp = self.fresh_temp();
                    self.push(Instr::SysNew { dst: tmp, size });
                    let ptr_ty = self.designator_ir_type(target).unwrap_or(IrType::UntaggedPtr(Box::new(IrType::Opaque("sysnew".to_string()))));
                    let target_addr = self.designator_addr(target);
                    self.push(Instr::Store { addr: target_addr, value: IrValue::Temp(tmp, ptr_ty) });
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    fn system_intrinsic(&self, des: &Designator) -> Option<&'static str> {
        let module = des.base.module.as_deref()?;
        if des.selectors.len() != 1 || !self.system_qualifiers.iter().any(|item| item == module) {
            return None;
        }
        match des.base.name.as_str() {
            "ADR" => Some("ADR"),
            "VAL" => Some("VAL"),
            "LSH" => Some("LSH"),
            "ROT" => Some("ROT"),
            "TYP" => Some("TYP"),
            "GET" => Some("GET"),
            "PUT" => Some("PUT"),
            "MOVE" => Some("MOVE"),
            "NEW" => Some("NEW"),
            _ => None,
        }
    }

    fn ir_type_from_type_arg(&self, expr: &Expr) -> Option<IrType> {
        let Expr::Designator(des) = expr else {
            return None;
        };
        if let Some(module) = &des.base.module {
            return Some(IrType::Named(format!("{module}.{}", des.base.name)));
        }
        match des.base.name.as_str() {
            "BOOLEAN" => Some(IrType::Bool),
            "BYTE" => Some(IrType::U8),
            "CHAR" => Some(IrType::Char),
            "SHORTCHAR" => Some(IrType::ShortChar),
            "INTEGER" | "LONGINT" => Some(IrType::I64),
            "SHORTINT" => Some(IrType::I16),
            "REAL" => Some(IrType::F64),
            "SHORTREAL" => Some(IrType::F32),
            _ => Some(IrType::Named(des.base.name.clone())),
        }
    }

    fn designator_ir_type(&self, des: &Designator) -> Option<IrType> {
        let des = self.normalize_designator(des);
        let mut ty = self.base_symbol_ir_type(&des.base)?;

        for selector in &des.selectors {
            ty = match (selector, ty) {
                (Selector::Dereference, IrType::Ptr(inner)) => *inner,
                (Selector::Dereference, IrType::UntaggedPtr(inner)) => *inner,
                (Selector::Index(_), IrType::Ptr(inner)) => *inner,
                (Selector::Index(_), IrType::UntaggedPtr(inner)) => *inner,
                (Selector::Field(fname), ref base_ty) => {
                    // Look up the field type in the resolved record, if possible.
                    if let Some(record_ty) = self.resolve_base_record_type(base_ty) {
                        let flat = Self::flatten_record_fields(record_ty);
                        if let Some((_, field_sem_ty)) = flat.iter().find(|(n, _)| n == fname) {
                            map_semantic_type(field_sem_ty)
                        } else {
                            IrType::Opaque(format!("field:{fname}"))
                        }
                    } else {
                        IrType::Opaque(format!("field:{fname}"))
                    }
                }
                (_, other) => other,
            };
        }

        Some(ty)
    }
}

// == Statement lowering ==

impl<'m> LowerCtx<'m> {
    fn lower_statements(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            self.lower_statement(stmt);
        }
    }

    fn lower_inc_dec_statement(&mut self, des: &Designator) -> bool {
        if des.base.module.is_some() || des.selectors.len() != 1 {
            return false;
        }

        let (target, delta_arg) = match &des.selectors[0] {
            Selector::Call(args) => {
                let Some(Expr::Designator(target)) = args.first() else {
                    return false;
                };
                (target.clone(), args.get(1))
            }
            Selector::AmbiguousParen(qual) => {
                (
                    Designator {
                        span: qual.span,
                        base: qual.clone(),
                        selectors: Vec::new(),
                    },
                    None,
                )
            }
            _ => return false,
        };

        let op = match des.base.name.as_str() {
            "INC" => BinOp::Add,
            "DEC" => BinOp::Sub,
            _ => return false,
        };

        let ty = self.designator_ir_type(&target).unwrap_or(IrType::I64);
        let addr = self.designator_addr(&target);
        let current_tmp = self.fresh_temp();
        self.push(Instr::Load {
            dst: current_tmp,
            addr: addr.clone(),
            ty: ty.clone(),
        });
        let delta = delta_arg
            .map(|expr| self.lower_expr(expr))
            .unwrap_or_else(|| self.const_one(&ty));
        let next_tmp = self.fresh_temp();
        self.push(Instr::BinOp {
            dst: next_tmp,
            op,
            left: IrValue::Temp(current_tmp, ty.clone()),
            right: delta,
            ty: ty.clone(),
        });
        self.push(Instr::Store {
            addr,
            value: IrValue::Temp(next_tmp, ty),
        });
        true
    }

    fn const_one(&self, ty: &IrType) -> IrValue {
        match ty {
            IrType::F32 | IrType::F64 => IrValue::ConstReal(1.0, ty.clone()),
            _ => IrValue::ConstInt(1, ty.clone()),
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
                if !self.lower_inc_dec_statement(designator)
                    && !self.lower_system_statement(designator)
                {
                    let _ = self.lower_designator(designator);
                }
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
            IrType::Ref(Box::new(IrType::I64)),
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
        self.push(Instr::Load { dst: var_t, addr: var_addr.clone(), ty: IrType::I64 });
        let end_val = self.lower_expr(end);
        let cmp_t = self.fresh_temp();
        self.push(Instr::BinOp {
            dst: cmp_t,
            op: BinOp::Le,
            left: IrValue::Temp(var_t, IrType::I64),
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
        self.push(Instr::Load { dst: var_t2, addr: var_addr.clone(), ty: IrType::I64 });
        let step_val = step
            .map(|s| self.lower_expr(s))
            .unwrap_or(IrValue::ConstInt(1, IrType::I64));
        let new_t = self.fresh_temp();
        self.push(Instr::BinOp {
            dst: new_t,
            op: BinOp::Add,
            left: IrValue::Temp(var_t2, IrType::I64),
            right: step_val,
            ty: IrType::I64,
        });
        self.push(Instr::Store {
            addr: var_addr,
            value: IrValue::Temp(new_t, IrType::I64),
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
    system_qualifiers: Vec<String>,
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
        {
            let mut symbols = sema_proc.local_symbols.clone();
            symbols.extend_from_slice(module_symbols);
            symbols
        },
        system_qualifiers,
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

    ctx.proc.prune_unreachable();
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
                .map(|ast_proc| lower_procedure(
                    sema_proc,
                    ast_proc,
                    ast.imports
                        .iter()
                        .filter(|item| item.name == "SYSTEM")
                        .flat_map(|item| {
                            let mut names = vec![item.name.clone()];
                            if let Some(alias) = &item.alias {
                                names.push(alias.clone());
                            }
                            names
                        })
                        .collect(),
                    &sema.symbols,
                ))
        })
        .collect();

    IrModule {
        name: sema.name.clone(),
        imports: sema.imports.clone(),
        globals,
        procedures,
        named_types: collect_named_types(&sema.symbols),
    }
}

/// Collect all TYPE declarations in the module that are records.
///
/// Returns a map from simple type name to the flattened ordered field list
/// `(field_name, IrType)`, with inherited fields from the base chain appearing first.
fn collect_named_types(
    module_symbols: &[SemanticSymbol],
) -> std::collections::HashMap<String, Vec<(String, IrType)>> {
    use newcp_sema::SymbolKind;
    let mut map = std::collections::HashMap::new();
    for sym in module_symbols {
        if sym.kind != SymbolKind::Type {
            continue;
        }
        if let Some(sem_ty) = &sym.declared_type {
            let flat = flatten_fields_deep(sem_ty, module_symbols);
            if !flat.is_empty() {
                map.insert(sym.name.clone(), flat);
            }
        }
    }
    map
}

/// Flatten all fields (including inherited ones) for a record-like SemanticType.
fn flatten_fields_deep(
    ty: &SemanticType,
    module_symbols: &[SemanticSymbol],
) -> Vec<(String, IrType)> {
    let (base, fields) = match ty {
        SemanticType::Record { base, fields, .. } => (base.as_deref(), fields.as_slice()),
        SemanticType::Named { name, module: None, .. } => {
            // Resolve local named type.
            if let Some(sym) = module_symbols.iter().find(|s| s.name == *name) {
                if let Some(resolved) = &sym.declared_type {
                    return flatten_fields_deep(resolved, module_symbols);
                }
            }
            return Vec::new();
        }
        _ => return Vec::new(),
    };
    let mut result = Vec::new();
    if let Some(parent) = base {
        result.extend(flatten_fields_deep(parent, module_symbols));
    }
    for field in fields {
        let ir_ty = map_semantic_type(&field.ty);
        for name in &field.names {
            result.push((name.clone(), ir_ty.clone()));
        }
    }
    result
}
