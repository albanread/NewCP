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
        SemanticType::Array { lengths, element_type, untagged } => {
            let elem_ir = map_semantic_type(element_type);
            if lengths.is_empty() {
                // Open array (VAR parameter, no explicit length) — lower as pointer to element.
                if *untagged {
                    IrType::UntaggedPtr(Box::new(elem_ir))
                } else {
                    IrType::Ptr(Box::new(elem_ir))
                }
            } else {
                // Fixed-length array — build nested IrType::Array from innermost out.
                // e.g. ARRAY 10, 20 OF INTEGER  →  [10 x [20 x i64]]
                // We start from the innermost dimension (last length) and wrap outward.
                // However, in sema the element_type is already the element (not a nested array
                // for multi-dim syntax), so we build [len[n-1] x (... [len[0] x elem])].
                let mut result = elem_ir;
                for len_str in lengths.iter().rev() {
                    let len: u64 = len_str.parse().unwrap_or(0);
                    result = IrType::Array { element: Box::new(result), len };
                }
                result
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
    /// Type overrides pushed by WITH arms so field access resolves against the guard type.
    with_type_overrides: Vec<(String, IrType)>,
    /// The unqualified name of the enclosing (outer) procedure, if any.
    /// Used when rewriting calls to nested procedures (mangling callee name).
    outer_proc_name: Option<String>,
    /// For each local procedure name, the list of upvalue (name, type) pairs that
    /// must be prepended as Ref arguments at every call site.
    nested_proc_upvalues: std::collections::HashMap<String, Vec<(String, SemanticType)>>,
    /// For each local procedure name, its return IrType (used for correct call typing).
    nested_proc_return_types: std::collections::HashMap<String, IrType>,
    /// Cache of already-parsed-and-analysed imported modules, keyed by module name.
    import_cache: std::collections::HashMap<String, SemanticModule>,
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
            with_type_overrides: Vec::new(),
            outer_proc_name: None,
            nested_proc_upvalues: std::collections::HashMap::new(),
            nested_proc_return_types: std::collections::HashMap::new(),
            import_cache: std::collections::HashMap::new(),
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

    /// If `type_name` is a named pointer alias, return the concrete IR pointer type.
    ///
    /// E.g. `DataPtr = POINTER TO Data` → `Some(IrType::Ptr(Named("Data")))`.
    fn resolve_named_as_ptr_ir_type(&self, type_name: &str) -> Option<IrType> {
        let ty = self
            .symbols
            .iter()
            .rev()
            .chain(self.module_symbols.iter().rev())
            .find(|sym| sym.kind == SymbolKind::Type && sym.name == type_name)
            .and_then(|sym| sym.declared_type.as_ref())?;
        match ty {
            SemanticType::Pointer { target, untagged } => {
                let inner = map_semantic_type(target);
                Some(if *untagged {
                    IrType::UntaggedPtr(Box::new(inner))
                } else {
                    IrType::Ptr(Box::new(inner))
                })
            }
            SemanticType::Named { module: None, name, .. } if name != type_name => {
                self.resolve_named_as_ptr_ir_type(name)
            }
            _ => None,
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
    fn base_symbol_ir_type(&self, qual: &QualIdent) -> Option<IrType> {
        // WITH-body overrides take priority so field access uses the narrowed type.
        if let Some((_, ty)) = self.with_type_overrides.iter().rev().find(|(n, _)| n == &qual.name) {
            return Some(ty.clone());
        }
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

    /// Flatten record fields for an IR type, including support for dot-qualified
    /// imported types like `"TypeExt.Bird"`.
    ///
    /// Strips pointer/ref wrappers and resolves the inner record's fields.
    fn flatten_fields_for_ir_type(&mut self, ir_ty: &IrType) -> Vec<(String, SemanticType)> {
        let mut cursor = ir_ty;
        loop {
            match cursor {
                IrType::Ptr(inner) | IrType::UntaggedPtr(inner) | IrType::Ref(inner) => {
                    cursor = inner.as_ref();
                }
                IrType::Named(n) => {
                    if let Some((module, name)) = n.split_once('.') {
                        return self.flatten_imported_record_fields(module, name);
                    }
                    // For local named types, use flatten_sem_type_fields which resolves
                    // Named base types (e.g. `Bird RECORD (Animal)` where Animal is Named).
                    let sem_ty = self
                        .symbols
                        .iter()
                        .rev()
                        .chain(self.module_symbols.iter())
                        .find(|sym| sym.kind == SymbolKind::Type && sym.name == n.as_str())
                        .and_then(|s| s.declared_type.as_ref());
                    return Self::flatten_sem_type_fields(sem_ty, self.module_symbols);
                }
                _ => return Vec::new(),
            }
        }
    }

    /// Load an imported module and flatten the fields of the named record type within it,
    /// including inherited fields from the base chain.
    fn flatten_imported_record_fields(&mut self, module: &str, type_name: &str) -> Vec<(String, SemanticType)> {
        let sema = load_cached_import(module, &mut self.import_cache);
        let Some(sema) = sema else { return Vec::new() };
        let Some(sym) = sema.symbols.iter().find(|s| s.name == type_name && s.kind == SymbolKind::Type) else {
            return Vec::new();
        };
        Self::flatten_sem_type_fields(sym.declared_type.as_ref(), &sema.symbols)
    }

    /// Recursively flatten a `SemanticType` (owned reference) with optional module symbols.
    fn flatten_sem_type_fields(
        ty: Option<&SemanticType>,
        module_symbols: &[SemanticSymbol],
    ) -> Vec<(String, SemanticType)> {
        let Some(ty) = ty else { return Vec::new() };
        match ty {
            SemanticType::Record { base, fields, .. } => {
                let mut result = Vec::new();
                if let Some(base_ty) = base {
                    match base_ty.as_ref() {
                        SemanticType::Named { module: None, name, .. } => {
                            // Local base — look up in the same module_symbols.
                            let sym = module_symbols.iter().find(|s| s.name == *name);
                            result.extend(Self::flatten_sem_type_fields(
                                sym.and_then(|s| s.declared_type.as_ref()),
                                module_symbols,
                            ));
                        }
                        SemanticType::Named { module: Some(m), name, .. } => {
                            // Cross-module base.
                            let path = Path::new("Mod").join(format!("{m}.cp"));
                            if let Ok(base_ast) = read_module_ast(&path) {
                                let base_sema = analyze_module_ast(&base_ast);
                                let sym = base_sema.symbols.iter().find(|s| s.name == *name);
                                let base_fields = Self::flatten_sem_type_fields(
                                    sym.and_then(|s| s.declared_type.as_ref()),
                                    &base_sema.symbols,
                                );
                                result.extend(base_fields);
                            }
                        }
                        other => {
                            result.extend(Self::flatten_record_fields(other));
                        }
                    }
                }
                for field in fields {
                    for name in &field.names {
                        result.push((name.clone(), field.ty.clone()));
                    }
                }
                result
            }
            SemanticType::Named { module: None, name, .. } => {
                let sym = module_symbols.iter().find(|s| s.name == *name);
                Self::flatten_sem_type_fields(sym.and_then(|s| s.declared_type.as_ref()), module_symbols)
            }
            _ => Vec::new(),
        }
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
            Expr::Designator(des) => self.lower_lang_builtin_expr(des)
                .or_else(|| self.lower_system_expr(des))
                .unwrap_or_else(|| self.lower_designator(des)),
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
        // Check for a bound procedure call: obj.Method(args)
        // Pattern: selectors end with [Field(method_name), Call(args)] and
        // method_name resolves to a method (not a data field) on the receiver type.
        if let Some(result) = self.lower_bound_proc_call_expr(des) {
            return result;
        }

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
            // Detect a call to a local nested procedure: no module qualifier and
            // the name maps to a local Procedure symbol in this proc's scope.
            // Rewrite to the flat qualified name and prepend upvalue addr args.
            let is_local_nested_proc = module_opt.is_none()
                && self.nested_proc_upvalues.contains_key(&base_name);

            let (callee_name, upvalue_args): (String, Vec<IrValue>) = if is_local_nested_proc {
                let outer = self.outer_proc_name.as_deref().unwrap_or("");
                let flat_name = format!("{outer}_{base_name}");
                let upvalues = self.nested_proc_upvalues[&base_name].clone();
                let uv_args: Vec<IrValue> = upvalues
                    .iter()
                    .map(|(uv_name, uv_ty)| {
                        IrValue::GlobalRef(uv_name.clone(), IrType::Ref(Box::new(map_semantic_type(uv_ty))))
                    })
                    .collect();
                (flat_name, uv_args)
            } else {
                (base_name.clone(), Vec::new())
            };

            let callee = match module_opt {
                Some(m) => IrValue::ImportRef(m, callee_name, final_ty.clone()),
                None => IrValue::GlobalRef(callee_name, final_ty.clone()),
            };

            match des.selectors.first() {
                Some(Selector::Call(args)) => {
                    let mut args_lowered = upvalue_args;
                    args_lowered.extend(self.lower_call_args(&callee, args));
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
                    let mut args_lowered = upvalue_args;
                    args_lowered.extend(self.lower_call_args(&callee, &[arg]));
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

    /// Attempt to lower a bound procedure call: `obj.Method(args)` or
    /// `ptr.Method(args)` (pointer receiver, implicit deref).
    ///
    /// Returns `None` if the designator is not a method call and normal
    /// lowering should continue.
    fn lower_bound_proc_call_expr(&mut self, des: &Designator) -> Option<IrValue> {
        // Pattern: selectors end with [.., Field(method_name), Call(args)].
        let selectors = &des.selectors;
        let n = selectors.len();
        if n < 2 {
            return None;
        }
        let (Selector::Field(method_name), Selector::Call(call_args)) =
            (&selectors[n - 2], &selectors[n - 1])
        else {
            return None;
        };

        // Resolve the receiver designator (everything before the last two selectors).
        // We need to know the RECORD type of the receiver so we can look up the slot.
        let prefix_des = Designator {
            span: des.span,
            base: des.base.clone(),
            selectors: selectors[..n - 2].to_vec(),
        };
        let prefix_ty = self.designator_ir_type(&prefix_des)?;

        // Strip pointer/ref wrappers to get the Named type.
        fn inner_named(ty: &IrType) -> Option<&str> {
            match ty {
                IrType::Named(n) => Some(n.as_str()),
                IrType::Ptr(inner) | IrType::UntaggedPtr(inner) | IrType::Ref(inner) => {
                    inner_named(inner)
                }
                _ => None,
            }
        }
        let type_qualified = inner_named(&prefix_ty)?;

        // Strip module qualifier for local lookup (cross-module dispatch deferred).
        let type_local_name = if let Some((_, local)) = type_qualified.split_once('.') {
            local
        } else {
            type_qualified
        };

        // Check that method_name is actually a METHOD (not a data field) of this type.
        // Use the sema symbol table to look at Record.methods.
        let slot = method_slot_in_vtable(type_local_name, method_name, self.module_symbols)?;

        // Lower the receiver.  For pointer types, load the pointer first; for Ref types
        // the address already IS the right thing.  We produce the object pointer (ptr).
        let receiver_ptr: IrValue = {
            let addr = self.designator_addr(&prefix_des);
            match addr.ty() {
                // addr is Ref(Ptr(...)) or Ref(Named(...)) — load to get the ptr value
                IrType::Ref(inner) if matches!(inner.as_ref(), IrType::Ptr(_) | IrType::Named(_)) => {
                    let t = self.fresh_temp();
                    let obj_ty = *inner;
                    self.push(Instr::Load { dst: t, addr, ty: obj_ty.clone() });
                    IrValue::Temp(t, obj_ty)
                }
                // addr is already a pointer-ish — use it directly
                _ => addr,
            }
        };

        // Build the call args: explicit args only (receiver is carried in MethodCall::descriptor,
        // and emit_method_call prepends it as the first LLVM argument).
        let mut lowered_args: Vec<IrValue> = vec![];
        for arg in call_args {
            lowered_args.push(self.lower_expr(arg));
        }

        // Look up the return type from the method's module-level symbol.
        let ret_ty = self
            .module_symbols
            .iter()
            .find(|s| {
                s.kind == SymbolKind::Procedure
                    && s.name == *method_name
                    && s.declared_type.as_ref().and_then(|t| {
                        if let SemanticType::Procedure(pt) = t { pt.receiver.as_deref() } else { None }
                    }).and_then(|r| match r {
                        SemanticType::Named { name, .. } => Some(name.as_str()),
                        _ => None,
                    }) == Some(type_local_name)
            })
            .and_then(|s| {
                if let Some(SemanticType::Procedure(pt)) = &s.declared_type {
                    Some(pt.result_type.as_ref().map(|t| map_semantic_type(t)).unwrap_or(IrType::Void))
                } else {
                    None
                }
            })
            .unwrap_or(IrType::Void);

        if ret_ty == IrType::Void {
            self.push(Instr::MethodCall {
                dst: None,
                descriptor: receiver_ptr,
                slot,
                args: lowered_args,
                ret_ty: IrType::Void,
            });
            // Return something innocuous; callers that use the result will get void.
            return Some(IrValue::ConstBool(false));
        }

        let t = self.fresh_temp();
        self.push(Instr::MethodCall {
            dst: Some(t),
            descriptor: receiver_ptr,
            slot,
            args: lowered_args,
            ret_ty: ret_ty.clone(),
        });
        Some(IrValue::Temp(t, ret_ty))
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
                        // Determine the concrete type to load and use as the GEP base.
                        // We only emit an IR Load here when the variable holds a *pointer* that
                        // must be dereferenced to reach the struct.  Two sub-cases:
                        //   Ref(Ptr|UntaggedPtr)          — local/global pointer variable
                        //   Ref(Ref(Ptr|UntaggedPtr))     — VAR param whose value is a pointer
                        //   Ref(Named("T")) / Ref(Ref(Named("T"))) where T is a pointer alias
                        //
                        // We do NOT emit a Load for Ref(Named) or Ref(Ref(Named)) where Named
                        // is a plain record type: in that case the LLVM `ref_param_slots`
                        // mechanism already loads the one-level indirection in `resolve_pointer`,
                        // and emitting an extra Load here causes a spurious double-dereference.
                        let effective_ty: Option<IrType> =
                            if matches!(inner.as_ref(), IrType::Ptr(_) | IrType::UntaggedPtr(_)) {
                                // Ref(Ptr): local/global pointer variable — load to get pointee ptr
                                Some(inner.as_ref().clone())
                            } else if let IrType::Ref(inner2) = inner.as_ref() {
                                // Ref(Ref(...)): VAR param — only load when the value is a pointer
                                if matches!(inner2.as_ref(), IrType::Ptr(_) | IrType::UntaggedPtr(_)) {
                                    Some(inner.as_ref().clone())
                                } else if let IrType::Named(n) = inner2.as_ref() {
                                    // VAR param of a pointer type alias
                                    self.resolve_named_as_ptr_ir_type(n)
                                        .map(|p| IrType::Ref(Box::new(p)))
                                } else {
                                    // VAR plain record param — ref_param_slots handles indirection
                                    None
                                }
                            } else if let IrType::Named(n) = inner.as_ref() {
                                self.resolve_named_as_ptr_ir_type(n)
                            } else {
                                None
                            };
                        if let Some(loaded_ty) = effective_ty {
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

                    let flat_fields = self.flatten_fields_for_ir_type(&base_ty);
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

                    let unresolved = IrType::Opaque(format!("field:{fname}"));
                    addr = IrValue::GlobalRef(
                        format!("field:{fname}"),
                        IrType::Ref(Box::new(unresolved)),
                    );
                }
                Selector::Index(index_exprs) => {
                    // For each index expression in the selector (e.g. `a[i, j]` has two),
                    // emit an IndexGep instruction and update `addr`.
                    for index_expr in index_exprs {
                        // Determine the element type and whether we need to load a pointer first.
                        let addr_ty = addr.ty();
                        let (gep_base, elem_ty, maybe_len) = match &addr_ty {
                            // Ref(Array { element, len }) — inline array; the ref IS the array start.
                            IrType::Ref(inner) => match inner.as_ref() {
                                IrType::Array { element, len } => {
                                    (addr.clone(), *element.clone(), Some(*len))
                                }
                                // Ref(Ptr(elem)) or Ref(UntaggedPtr(elem)) — need to load the pointer first.
                                IrType::Ptr(elem) | IrType::UntaggedPtr(elem) => {
                                    let loaded_ptr_ty = inner.as_ref().clone();
                                    let t = self.fresh_temp();
                                    self.push(Instr::Load {
                                        dst: t,
                                        addr: addr.clone(),
                                        ty: loaded_ptr_ty.clone(),
                                    });
                                    (IrValue::Temp(t, loaded_ptr_ty), *elem.clone(), None)
                                }
                                // Ref(Named) — try to resolve the named type's element.
                                IrType::Named(_) => {
                                    // Fall back: just use addr and opaque element.
                                    (addr.clone(), IrType::Opaque("array-elem".to_string()), None)
                                }
                                _ => (addr.clone(), IrType::Opaque("array-elem".to_string()), None),
                            },
                            // Already a pointer type (e.g. from a loaded pointer).
                            IrType::Ptr(elem) | IrType::UntaggedPtr(elem) => {
                                (addr.clone(), *elem.clone(), None)
                            }
                            _ => (addr.clone(), IrType::Opaque("array-elem".to_string()), None),
                        };

                        // Lower the index expression to an integer value.
                        let idx_val = self.lower_expr(index_expr);

                        // Optional bounds check: emit CondBr → trap if index ≥ len.
                        if let Some(len) = maybe_len {
                            // Use U64 for both operands so the comparison is unsigned (ult),
                            // catching both negative indices and indices ≥ len in one test.
                            let len_val = IrValue::ConstInt(len as i128, IrType::U64);
                            let idx_cast = {
                                let t = self.fresh_temp();
                                self.push(Instr::BitCast {
                                    dst: t,
                                    value: idx_val.clone(),
                                    ty: IrType::U64,
                                });
                                IrValue::Temp(t, IrType::U64)
                            };
                            // in_bounds = (idx as u64) < len  — unsigned, rejects negatives too
                            let ok_block = self.alloc_block();
                            let fail_block = self.alloc_block();
                            let cmp = self.fresh_temp();
                            self.push(Instr::BinOp {
                                dst: cmp,
                                op: BinOp::Lt,
                                left: idx_cast,
                                right: len_val,
                                ty: IrType::Bool,
                            });
                            self.set_term(Terminator::CondBr {
                                cond: IrValue::Temp(cmp, IrType::Bool),
                                true_target: ok_block,
                                false_target: fail_block,
                            });
                            self.switch_to(fail_block);
                            self.set_term(Terminator::Trap { kind: TrapKind::ArrayBounds });
                            self.switch_to(ok_block);
                        }

                        // Emit the IndexGep.
                        let t = self.fresh_temp();
                        self.push(Instr::IndexGep {
                            dst: t,
                            base: gep_base,
                            index: idx_val,
                            element_ty: elem_ty.clone(),
                        });
                        addr = IrValue::Temp(t, IrType::Ref(Box::new(elem_ty)));
                    }
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

    fn callee_return_type(&mut self, callee: &IrValue) -> IrType {
        match self.callee_procedure_type(callee) {
            Some(proc_ty) => proc_ty
                .result_type
                .as_ref()
                .map(|ty| map_semantic_type(ty.as_ref()))
                .unwrap_or(IrType::Void),
            _ => {
                // Check if this is a lifted nested procedure call.
                if let IrValue::GlobalRef(name, _) = callee {
                    let outer = self.outer_proc_name.as_deref().unwrap_or("");
                    if let Some(inner) = name.strip_prefix(&format!("{outer}_")) {
                        if let Some(ret) = self.nested_proc_return_types.get(inner) {
                            return ret.clone();
                        }
                    }
                }
                IrType::Opaque("call-result".to_string())
            }
        }
    }

    fn callee_procedure_type(&mut self, callee: &IrValue) -> Option<newcp_sema::ProcedureType> {
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
        &mut self,
        module: &str,
        name: &str,
    ) -> Option<newcp_sema::ProcedureType> {
        let sema = load_cached_import(module, &mut self.import_cache)?;
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

    /// Handle standard CP language built-in functions as expressions:
    ///   ODD(x)     → (x & 1) != 0
    ///   ASH(x, n)  → Instr::Ash (shl for n≥0, arithmetic-shr for n<0)
    ///   ABS(x)     → x ≥ 0 ? x : -x  (emitted as BinOp sequence)
    ///
    /// Returns `None` when the designator is not a recognised builtin.
    fn lower_lang_builtin_expr(&mut self, des: &Designator) -> Option<IrValue> {
        // Only bare (unqualified) single-argument calls.
        if des.base.module.is_some() {
            return None;
        }
        let args: Vec<Expr> = match des.selectors.last()? {
            Selector::Call(args) => args.clone(),
            Selector::AmbiguousParen(qual) => vec![Expr::Designator(Designator {
                span: qual.span,
                base: qual.clone(),
                selectors: Vec::new(),
            })],
            _ => return None,
        };

        match des.base.name.as_str() {
            "ODD" => {
                let x = self.lower_expr(args.first()?);
                let ty = x.ty();
                let masked_t = self.fresh_temp();
                self.push(Instr::BinOp {
                    dst: masked_t,
                    op: BinOp::And,
                    left: x,
                    right: IrValue::ConstInt(1, ty.clone()),
                    ty: ty.clone(),
                });
                let odd_t = self.fresh_temp();
                self.push(Instr::BinOp {
                    dst: odd_t,
                    op: BinOp::Ne,
                    left: IrValue::Temp(masked_t, ty.clone()),
                    right: IrValue::ConstInt(0, ty),
                    ty: IrType::Bool,
                });
                Some(IrValue::Temp(odd_t, IrType::Bool))
            }
            "ASH" => {
                let value = self.lower_expr(args.first()?);
                let shift = self.lower_expr(args.get(1)?);
                let ty = value.ty();
                let dst = self.fresh_temp();
                self.push(Instr::Ash { dst, value, shift, ty: ty.clone() });
                Some(IrValue::Temp(dst, ty))
            }
            _ => None,
        }
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

    fn designator_ir_type(&mut self, des: &Designator) -> Option<IrType> {
        let des = self.normalize_designator(des);
        let mut ty = self.base_symbol_ir_type(&des.base)?;

        for selector in &des.selectors {
            // When traversing into a Ref (VAR param / upvalue), the first selector
            // implicitly dereferences one level.
            if let IrType::Ref(inner) = ty {
                ty = *inner;
            }
            ty = match (selector, ty) {
                (Selector::Dereference, IrType::Ptr(inner)) => *inner,
                (Selector::Dereference, IrType::UntaggedPtr(inner)) => *inner,
                (Selector::Index(_), IrType::Ptr(inner)) => *inner,
                (Selector::Index(_), IrType::UntaggedPtr(inner)) => *inner,
                (Selector::Index(_), IrType::Array { element, .. }) => *element,
                (Selector::Field(fname), ref base_ty) => {
                    // Look up the field type in the resolved record, if possible.
                    let flat = self.flatten_fields_for_ir_type(base_ty);
                    if let Some((_, field_sem_ty)) = flat.iter().find(|(n, _)| n == fname) {
                        map_semantic_type(field_sem_ty)
                    } else {
                        IrType::Opaque(format!("field:{fname}"))
                    }
                }
                (_, other) => other,
            };
        }

        // For no-selector access to a Ref-typed symbol (VAR/upvalue param),
        // the value type is the inner type (dereferenced once).
        if des.selectors.is_empty() {
            if let IrType::Ref(inner) = ty {
                return Some(*inner);
            }
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

    /// Handle CP language built-in procedure calls:
    ///   NEW(ptr)        → Instr::New + Store
    ///   ASSERT(cond)    → CondBr to trap block
    ///   HALT(n)         → Terminator::Trap
    ///
    /// Returns `true` when the statement was handled.
    fn lower_builtin_statement(&mut self, des: &Designator) -> bool {
        // Only unqualified, single-call-selector designators.
        if des.base.module.is_some() {
            return false;
        }

        // Extract args from either a Call or AmbiguousParen selector.
        // AmbiguousParen wraps a single qualident; convert it to a single-element Expr vec.
        let args_call: Vec<Expr>;
        let args: &[Expr] = match des.selectors.first() {
            Some(Selector::Call(args)) => args.as_slice(),
            Some(Selector::AmbiguousParen(qual)) => {
                args_call = vec![Expr::Designator(Designator {
                    span: qual.span,
                    base: qual.clone(),
                    selectors: Vec::new(),
                })];
                &args_call
            }
            _ => return false,
        };

        match des.base.name.as_str() {
            "NEW" => {
                // NEW(ptr_var) — allocate a fresh heap record and store into ptr_var.
                let Some(Expr::Designator(target)) = args.first() else {
                    return false;
                };
                // Resolve the pointer alias to get the record type to allocate.
                let ptr_sym_ty = self.base_symbol_ir_type(&target.base)
                    .unwrap_or(IrType::Opaque("new-ptr".to_string()));
                let record_ty = match &ptr_sym_ty {
                    IrType::Ptr(inner) | IrType::UntaggedPtr(inner) => inner.as_ref().clone(),
                    IrType::Named(n) => {
                        // Pointer alias — unwrap to the target record type.
                        self.resolve_named_as_ptr_ir_type(n)
                            .and_then(|pt| match pt {
                                IrType::Ptr(inner) | IrType::UntaggedPtr(inner) => Some(*inner),
                                _ => None,
                            })
                            .unwrap_or(IrType::Opaque("new-target".to_string()))
                    }
                    other => other.clone(),
                };
                let dst = self.fresh_temp();
                self.push(Instr::New { dst, record_ty: record_ty.clone() });
                // Compute the concrete IR pointer type for storing back.
                let ptr_ir_ty = match &ptr_sym_ty {
                    IrType::Named(n) => self.resolve_named_as_ptr_ir_type(n)
                        .unwrap_or_else(|| IrType::Ptr(Box::new(record_ty))),
                    other => other.clone(),
                };
                let target_addr = self.designator_addr(target);
                self.push(Instr::Store {
                    addr: target_addr,
                    value: IrValue::Temp(dst, ptr_ir_ty),
                });
                true
            }
            "ASSERT" => {
                // ASSERT(cond [, error_code]) — trap if condition is false.
                let Some(cond_expr) = args.first() else {
                    return false;
                };
                let cond = self.lower_expr(cond_expr);
                let ok_block = self.alloc_block();
                let fail_block = self.alloc_block();
                self.set_term(Terminator::CondBr {
                    cond,
                    true_target: ok_block,
                    false_target: fail_block,
                });
                self.switch_to(fail_block);
                self.set_term(Terminator::Trap { kind: TrapKind::Assert });
                self.switch_to(ok_block);
                true
            }
            "HALT" => {
                // HALT(n) — immediate trap with the given code.
                let code = args.first()
                    .and_then(|e| if let Expr::Literal { value: newcp_parser::Literal::Integer(s), .. } = e {
                        s.parse::<i32>().ok()
                    } else {
                        None
                    })
                    .unwrap_or(0);
                self.set_term(Terminator::Trap { kind: TrapKind::Halt(code) });
                // Allocate a fresh unreachable block so the builder stays consistent.
                let dead = self.alloc_block();
                self.switch_to(dead);
                true
            }
            _ => false,
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
                    && !self.lower_builtin_statement(designator)
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

        if arms.is_empty() {
            if let Some(else_stmts) = else_branch {
                self.lower_statements(else_stmts);
            }
            self.set_term(Terminator::Br { target: merge_block });
            self.switch_to(merge_block);
            return;
        }

        // One label-test block per arm and one body block per arm.
        // The current block branches unconditionally to the first test block.
        let test_blocks: Vec<BlockId> = arms.iter().map(|_| self.alloc_block()).collect();
        let body_blocks: Vec<BlockId> = arms.iter().map(|_| self.alloc_block()).collect();
        let else_block = if else_branch.is_some() { Some(self.alloc_block()) } else { None };
        let trap_block = self.alloc_block();
        let final_miss = else_block.unwrap_or(trap_block);

        // Entry → first test block.
        self.set_term(Terminator::Br { target: test_blocks[0] });

        for (arm_idx, arm) in arms.iter().enumerate() {
            let test_block = test_blocks[arm_idx];
            let body_block = body_blocks[arm_idx];
            let miss = if arm_idx + 1 < arms.len() {
                test_blocks[arm_idx + 1]
            } else {
                final_miss
            };

            // Emit label comparisons in the test block.
            self.switch_to(test_block);
            self.lower_case_labels(&subject_val.clone(), &arm.labels, body_block, miss);

            // Emit arm body.
            self.switch_to(body_block);
            self.lower_statements(&arm.body);
            self.set_term(Terminator::Br { target: merge_block });
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
                    var_name.clone(),
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
                    ty: guard_ty.clone(),
                    true_target: body_block,
                    false_target: next_block,
                });

                self.switch_to(body_block);
                // Within the body, treat `var_name` as having the narrowed pointer type so
                // field access resolves against the guard record type (e.g. Bird, not Animal).
                let guard_ref_ty = IrType::Ref(Box::new(guard_ty));
                self.with_type_overrides.push((var_name.clone(), guard_ref_ty));
                self.lower_statements(&arm.body);
                self.with_type_overrides.pop();
                self.set_term(Terminator::Br { target: merge_block });
            } else {
                // ELSE arm -- always taken.
                self.set_term(Terminator::Br { target: body_block });
                self.switch_to(body_block);
                self.lower_statements(&arm.body);
                self.set_term(Terminator::Br { target: merge_block });
            }

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
    all_sema_procs: &[SemanticProcedure],
) -> IrProcedure {
    use newcp_sema::SymbolKind;

    // Build LLVM parameter list.  If this is a bound procedure (receiver present),
    // the receiver is prepended as a direct *object pointer* (Ptr), not a VAR (Ref).
    // The caller always passes the heap pointer directly; Ref would add a spurious
    // extra dereference on every field access in the callee.
    let receiver_param: Option<(String, IrType)> = sema_proc
        .local_symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Receiver)
        .and_then(|s| {
            let recv_ty = s.declared_type.as_ref().map(map_semantic_type)?;
            Some((s.name.clone(), IrType::Ptr(Box::new(recv_ty))))
        });

    // Nested procedures: captured outer variables are prepended as implicit Ref params,
    // exactly like VAR params.  This lets the LLVM backend's ref_param_slots mechanism
    // handle the extra indirection transparently.
    let upvalue_params: Vec<(String, IrType)> = sema_proc
        .upvalues
        .iter()
        .map(|(name, ty)| (name.clone(), IrType::Ref(Box::new(map_semantic_type(ty)))))
        .collect();

    let mut params: Vec<(String, IrType)> = upvalue_params;
    params.extend(receiver_param.iter().cloned());
    params.extend(sema_proc
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
        }));

    let ret_ty = sema_proc
        .signature
        .result_type
        .as_ref()
        .map(|t| map_semantic_type(t))
        .unwrap_or(IrType::Void);

    // Qualified name: nested procs already have "Outer_Inner" in sema_proc.name.
    // Bound procedures get "ReceiverType_MethodName".
    let proc_name = if sema_proc.parent_proc.is_some() {
        // Already qualified in sema.
        sema_proc.name.clone()
    } else if let Some(recv_ty) = &sema_proc.signature.receiver {
        let recv_name = match recv_ty.as_ref() {
            SemanticType::Named { name, .. } => name.clone(),
            _ => "Unknown".to_string(),
        };
        format!("{recv_name}_{}", sema_proc.name)
    } else {
        sema_proc.name.clone()
    };

    // The unqualified name of this procedure (for nested-proc call mangling).
    // For "Outer_Inner", outer_name = "Outer"; for top-level "Outer", outer_name = "Outer".
    let outer_name_for_ctx = sema_proc.parent_proc
        .as_deref()
        .unwrap_or(&sema_proc.name)
        .to_string();

    // Map of local proc name → its upvalues and return type, for rewriting call sites.
    let nested_sema_procs: Vec<_> = all_sema_procs
        .iter()
        .filter(|p| p.parent_proc.as_deref() == Some(outer_name_for_ctx.as_str()))
        .collect();
    let nested_proc_upvalues: std::collections::HashMap<String, Vec<(String, SemanticType)>> =
        nested_sema_procs.iter()
            .map(|p| {
                let inner_name = p.name
                    .strip_prefix(&format!("{outer_name_for_ctx}_"))
                    .unwrap_or(&p.name)
                    .to_string();
                (inner_name, p.upvalues.clone())
            })
            .collect();
    let nested_proc_return_types: std::collections::HashMap<String, IrType> =
        nested_sema_procs.iter()
            .map(|p| {
                let inner_name = p.name
                    .strip_prefix(&format!("{outer_name_for_ctx}_"))
                    .unwrap_or(&p.name)
                    .to_string();
                let ret = p.signature.result_type.as_deref()
                    .map(map_semantic_type)
                    .unwrap_or(IrType::Void);
                (inner_name, ret)
            })
            .collect();

    let mut proc = IrProcedure::new(
        proc_name,
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
    ctx.outer_proc_name = Some(outer_name_for_ctx);
    ctx.nested_proc_upvalues = nested_proc_upvalues;
    ctx.nested_proc_return_types = nested_proc_return_types;

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

/// Load and semantically analyse an imported module, caching the result.
/// Subsequent calls with the same module name return the cached `SemanticModule`.
fn load_cached_import<'c>(
    module: &str,
    cache: &'c mut std::collections::HashMap<String, SemanticModule>,
) -> Option<&'c SemanticModule> {
    if !cache.contains_key(module) {
        let path = Path::new("Mod").join(format!("{module}.cp"));
        let ast = read_module_ast(&path).ok()?;
        let sema = analyze_module_ast(&ast);
        cache.insert(module.to_string(), sema);
    }
    cache.get(module)
}

pub fn lower_module(sema: &SemanticModule, ast: &ModuleAst) -> IrModule {
    use newcp_sema::SymbolKind;

    let mut import_cache: std::collections::HashMap<String, SemanticModule> = std::collections::HashMap::new();

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

    let system_qualifiers: Vec<String> = ast.imports
        .iter()
        .filter(|item| item.name == "SYSTEM")
        .flat_map(|item| {
            let mut names = vec![item.name.clone()];
            if let Some(alias) = &item.alias {
                names.push(alias.clone());
            }
            names
        })
        .collect();

    let procedures: Vec<IrProcedure> = sema
        .procedures
        .iter()
        .filter_map(|sema_proc| {
            // Match by name AND, for bound procedures, by receiver type to handle
            // multiple overloads with the same name but different receivers.
            let receiver_type_name: Option<&str> = sema_proc
                .signature
                .receiver
                .as_deref()
                .and_then(|rt| match rt {
                    SemanticType::Named { name, .. } => Some(name.as_str()),
                    _ => None,
                });

            ast_procs
                .iter()
                .find(|p| {
                    p.heading.name.name == sema_proc.name
                        && p.heading.receiver.as_ref().map(|r| r.ty.as_str()) == receiver_type_name
                })
                .map(|ast_proc| lower_procedure(
                    sema_proc,
                    ast_proc,
                    system_qualifiers.clone(),
                    &sema.symbols,
                    &sema.procedures,
                ))
        })
        .collect();

    // Lower nested procedures: their sema entries have parent_proc = Some("OuterName").
    // The AST for a nested proc lives inside the outer AST proc's body declarations.
    let nested_procedures: Vec<IrProcedure> = sema
        .procedures
        .iter()
        .filter(|sp| sp.parent_proc.is_some())
        .filter_map(|sema_nested| {
            let parent_name = sema_nested.parent_proc.as_deref()?;
            // Unqualified inner name: strip "ParentName_" prefix from the qualified name.
            let inner_name = sema_nested.name.strip_prefix(&format!("{parent_name}_"))?;
            // Find the outer AST proc.
            let parent_ast = ast_procs.iter().find(|p| p.heading.name.name == parent_name)?;
            // Find the nested AST proc inside the outer proc's body.
            let nested_ast = parent_ast
                .body
                .as_ref()?
                .declarations
                .iter()
                .filter_map(|d| match d { Declaration::Procedure(p) => Some(p), _ => None })
                .find(|p| p.heading.name.name == inner_name)?;
            Some(lower_procedure(
                sema_nested,
                nested_ast,
                system_qualifiers.clone(),
                &sema.symbols,
                &sema.procedures,
            ))
        })
        .collect();

    let mut procedures = procedures;
    procedures.extend(nested_procedures);

    let named_types = collect_named_types(&sema.name, &sema.imports, &sema.symbols, &mut import_cache);
    let (type_vtables, type_bases) = collect_type_vtables(&sema.name, &sema.symbols);

    IrModule {
        name: sema.name.clone(),
        imports: sema.imports.clone(),
        globals,
        procedures,
        named_types,
        type_vtables,
        type_bases,
    }
}

/// Collect vtable information for all record types in the module.
///
/// Returns two maps:
/// - `type_vtables`:  simple type name → ordered list of LLVM function names for each vtable slot.
///   The list represents the concrete implementations for objects of *exactly* that type:
///   inherited slots reuse the name of the override if present, otherwise the base implementation.
/// - `type_bases`:    simple type name → `Some("BaseTypeName")` or `None`.
fn collect_type_vtables(
    _module_name: &str,
    module_symbols: &[SemanticSymbol],
) -> (
    std::collections::HashMap<String, Vec<String>>,
    std::collections::HashMap<String, Option<String>>,
) {
    use newcp_sema::SymbolKind;
    let mut vtables: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut bases:   std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();

    for sym in module_symbols {
        if sym.kind != SymbolKind::Type {
            continue;
        }
        let SemanticType::Record { base, methods, .. } =
            sym.declared_type.as_ref().unwrap_or(&SemanticType::Nil)
        else {
            continue;
        };
        if methods.is_empty() && base.is_none() {
            continue; // plain record with no methods — no vtable
        }

        // Record direct base name (local only for now).
        let base_name: Option<String> = base.as_deref().and_then(|b| match b {
            SemanticType::Named { name, module: None, .. } => Some(name.clone()),
            _ => None,
        });
        bases.insert(sym.name.clone(), base_name.clone());

        // Build the vtable for this type.
        // Strategy: start from the base vtable (if any), then patch in any overrides.
        let mut vtable: Vec<String> = base_name
            .as_deref()
            .and_then(|bn| vtables.get(bn))
            .cloned()
            .unwrap_or_default();

        for method in methods {
            let llvm_name = format!("{}_{}", sym.name, method.name);
            if method.signature.is_new {
                // NEW method: extend the vtable.
                vtable.push(llvm_name);
            } else {
                // Override: find the existing slot (from base) and replace.
                let base_fn = base_name.as_deref()
                    .and_then(|bn| method_slot_in_vtable(bn, &method.name, module_symbols))
                    .map(|slot| slot as usize);
                if let Some(slot) = base_fn {
                    if slot < vtable.len() {
                        vtable[slot] = llvm_name;
                    }
                }
            }
        }

        if !vtable.is_empty() {
            vtables.insert(sym.name.clone(), vtable);
        }
    }

    (vtables, bases)
}

/// Find the vtable slot index of `method_name` in type `type_name` (within this module).
///
/// Returns `None` if the type or method is not found.
pub fn method_slot_in_vtable(
    type_name: &str,
    method_name: &str,
    module_symbols: &[SemanticSymbol],
) -> Option<u32> {
    use newcp_sema::SymbolKind;
    let sym = module_symbols.iter().find(|s| s.kind == SymbolKind::Type && s.name == type_name)?;
    let SemanticType::Record { base, methods, .. } = sym.declared_type.as_ref()? else {
        return None;
    };

    // Count how many slots the base type has.
    let base_slot_count: u32 = base
        .as_deref()
        .and_then(|b| match b {
            SemanticType::Named { name, module: None, .. } => {
                Some(count_vtable_slots(name, module_symbols))
            }
            _ => None,
        })
        .unwrap_or(0);

    // Check if the method is NEW in this type.
    let new_methods: Vec<&newcp_sema::MethodType> =
        methods.iter().filter(|m| m.signature.is_new).collect();
    if let Some(pos) = new_methods.iter().position(|m| m.name == method_name) {
        return Some(base_slot_count + pos as u32);
    }

    // Not NEW here — it's an override; delegate to the base.
    let base_name = base.as_deref().and_then(|b| match b {
        SemanticType::Named { name, module: None, .. } => Some(name.as_str()),
        _ => None,
    })?;
    method_slot_in_vtable(base_name, method_name, module_symbols)
}

/// Total number of vtable slots for a type (inherited + own NEW methods).
fn count_vtable_slots(type_name: &str, module_symbols: &[SemanticSymbol]) -> u32 {
    use newcp_sema::SymbolKind;
    let sym = match module_symbols.iter().find(|s| s.kind == SymbolKind::Type && s.name == type_name) {
        Some(s) => s,
        None => return 0,
    };
    let ty = match sym.declared_type.as_ref() {
        Some(ty) => ty,
        None => return 0,
    };
    let (base, methods) = match ty {
        SemanticType::Record { base, methods, .. } => (base, methods),
        _ => return 0,
    };
    let base_count: u32 = base
        .as_deref()
        .and_then(|b| match b {
            SemanticType::Named { name, module: None, .. } => Some(count_vtable_slots(name, module_symbols)),
            _ => None,
        })
        .unwrap_or(0);
    let own_new: u32 = methods.iter().filter(|m| m.signature.is_new).count() as u32;
    base_count + own_new
}

/// Collect all TYPE declarations in the module that are records.
///
/// Returns a map from simple type name to the flattened ordered field list
/// `(field_name, IrType)`, with inherited fields from the base chain appearing first.
fn collect_named_types(
    module_name: &str,
    imports: &[String],
    module_symbols: &[SemanticSymbol],
    import_cache: &mut std::collections::HashMap<String, SemanticModule>,
) -> std::collections::HashMap<String, Vec<(String, IrType)>> {
    use newcp_sema::SymbolKind;
    let mut map = std::collections::HashMap::new();

    // Collect local record types (stored under simple names).
    for sym in module_symbols {
        if sym.kind != SymbolKind::Type {
            continue;
        }
        if let Some(sem_ty) = &sym.declared_type {
            let flat = flatten_fields_deep(sem_ty, module_symbols);
            if !flat.is_empty() {
                // Also add with the module-qualified key so cross-module references work.
                let qualified = format!("{module_name}.{}", sym.name);
                map.insert(qualified, flat.clone());
                map.insert(sym.name.clone(), flat);
            }
        }
    }

    // Collect exported record types from imported modules, stored under "Module.Type" keys.
    for import_name in imports {
        // Load (or retrieve from cache), then clone to release the borrow so we can
        // pass &mut import_cache to the recursive flatten call below.
        let sema = match load_cached_import(import_name, import_cache) {
            Some(s) => s.clone(),
            None => continue,
        };
        for sym in &sema.symbols {
            if sym.kind != SymbolKind::Type || !sym.exported {
                continue;
            }
            if let Some(sem_ty) = &sym.declared_type {
                let flat = flatten_fields_deep_cross_module(sem_ty, &sema.symbols, import_name, import_cache);
                if !flat.is_empty() {
                    let key = format!("{import_name}.{}", sym.name);
                    map.insert(key, flat);
                }
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

/// Like `flatten_fields_deep` but handles cross-module base types (e.g. RECORD(TypeExt.Animal)).
fn flatten_fields_deep_cross_module(
    ty: &SemanticType,
    module_symbols: &[SemanticSymbol],
    current_module: &str,
    import_cache: &mut std::collections::HashMap<String, SemanticModule>,
) -> Vec<(String, IrType)> {
    let (base, fields) = match ty {
        SemanticType::Record { base, fields, .. } => (base.as_deref(), fields.as_slice()),
        SemanticType::Named { name, module: None, .. } => {
            if let Some(sym) = module_symbols.iter().find(|s| s.name == *name) {
                if let Some(resolved) = &sym.declared_type {
                    return flatten_fields_deep_cross_module(resolved, module_symbols, current_module, import_cache);
                }
            }
            return Vec::new();
        }
        SemanticType::Named { name, module: Some(m), .. } => {
            // Base type is in another module — load and recurse.
            if let Some(base_sema) = load_cached_import(m.as_str(), import_cache) {
                if let Some(sym) = base_sema.symbols.iter().find(|s| s.name == *name) {
                    if let Some(resolved) = &sym.declared_type {
                        // Clone to avoid borrow conflict after cache lookup
                        let resolved = resolved.clone();
                        let symbols: Vec<_> = base_sema.symbols.clone();
                        let m_str = m.clone();
                        let _ = base_sema;
                        return flatten_fields_deep_cross_module(&resolved, &symbols, &m_str, import_cache);
                    }
                }
            }
            return Vec::new();
        }
        _ => return Vec::new(),
    };
    let mut result = Vec::new();
    if let Some(parent) = base {
        result.extend(flatten_fields_deep_cross_module(parent, module_symbols, current_module, import_cache));
    }
    for field in fields {
        let ir_ty = map_semantic_type(&field.ty);
        for name in &field.names {
            result.push((name.clone(), ir_ty.clone()));
        }
    }
    result
}
