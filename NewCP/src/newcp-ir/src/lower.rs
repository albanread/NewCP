#![deny(clippy::unwrap_used)]

/// Lowering: Component Pascal AST (via SemanticModule + ModuleAst) -> IrModule/IrProcedure.
///
/// Design notes:
/// - The CFG *is* the IR; no separate TAC pass.
/// - Every logical RETURN compiles to StoreResult (if non-void) + Br(function_exit).
/// - The function_exit block emits the physical Ret (or RetVoid).
/// - EXIT inside a LOOP emits Br(loop_exit_target).
/// - WITH arms with a None guard are the ELSE arm.
/// - After all blocks are built, RPO is computed and stored on each block.
use std::cell::RefCell;
use std::path::{Path, PathBuf};

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
    procedure::{IrGlobal, IrModule, IrProcedure, LoweringDiagnostic},
    types::{IrType, RecordLayout},
};

thread_local! {
    static IMPORT_SEARCH_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub(crate) struct ImportSearchRootGuard {
    previous: Option<PathBuf>,
}

impl Drop for ImportSearchRootGuard {
    fn drop(&mut self) {
        IMPORT_SEARCH_ROOT.with(|root| {
            *root.borrow_mut() = self.previous.take();
        });
    }
}

pub(crate) fn push_import_search_root(path: &Path) -> ImportSearchRootGuard {
    let next_root = path.parent().map(Path::to_path_buf);
    let previous = IMPORT_SEARCH_ROOT.with(|root| {
        let mut root = root.borrow_mut();
        std::mem::replace(&mut *root, next_root)
    });

    ImportSearchRootGuard { previous }
}

// == Type mapping ==

/// Open-array param ABI: alongside the array pointer, the callee receives a
/// hidden `<name>$len: I64` parameter holding the array's element count.
/// `LEN(arr)` on an open-array param lowers to a load of this hidden slot.
/// This is the suffix used for the hidden param's name in `IrProcedure.params`.
pub(crate) const OPEN_ARRAY_LEN_SUFFIX: &str = "$len";

/// Returns true when `ty` is a Component Pascal open array
/// (i.e. `ARRAY OF T` with no explicit length).
pub(crate) fn is_open_array(ty: &SemanticType) -> bool {
    matches!(ty, SemanticType::Array { lengths, .. } if lengths.is_empty())
}

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
            // Special case: `POINTER TO ARRAY OF T` (open-array target).
            // The natural mapping `Ptr(Ptr(T))` (because open arrays
            // map as `Ptr(elem)` at the type level) gives one level of
            // indirection too many. The pointer should go directly to
            // the element data on the heap, so emit `Ptr(elem)` /
            // `UntaggedPtr(elem)` here.
            if let SemanticType::Array { lengths, element_type, untagged: arr_untagged } = target.as_ref() {
                if lengths.is_empty() {
                    let elem_ir = map_semantic_type(element_type);
                    if *untagged || *arr_untagged {
                        return IrType::UntaggedPtr(Box::new(elem_ir));
                    }
                    return IrType::Ptr(Box::new(elem_ir));
                }
            }
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

fn is_float_ir(ty: &IrType) -> bool { matches!(ty, IrType::F32 | IrType::F64) }

fn is_int_ir(ty: &IrType) -> bool {
    matches!(
        ty,
        IrType::I8 | IrType::I16 | IrType::I32 | IrType::I64
            | IrType::U8 | IrType::U16 | IrType::U32 | IrType::U64
    )
}

/// True when the IR type is a scalar numeric or character value that
/// `Instr::Cast` knows how to convert (sign/zero-extend, truncate, fp-cast).
/// Used to gate implicit return-value widening so we don't try to coerce
/// across pointers, records, arrays, etc.
fn is_scalar_castable(ty: &IrType) -> bool {
    matches!(
        ty,
        IrType::I8
            | IrType::I16
            | IrType::I32
            | IrType::I64
            | IrType::U8
            | IrType::U16
            | IrType::U32
            | IrType::U64
            | IrType::Bool
            | IrType::Char
            | IrType::ShortChar
            | IrType::F32
            | IrType::F64
    )
}

/// Single-argument `MIN(T)` / `MAX(T)`: extract the bounds of a basic type as a
/// constant. The argument is a bare type identifier (`LONGINT`, `INTEGER`,
/// `REAL`, `CHAR`, `SET`, ...). Returns `None` if the arg is not a
/// recognized type name.
fn min_max_one_arg(arg: &Expr, max: bool) -> Option<IrValue> {
    // The arg must be a bare designator naming a builtin type.
    let name = match arg {
        Expr::Designator(des)
            if des.base.module.is_none() && des.selectors.is_empty() =>
        {
            des.base.name.as_str()
        }
        _ => return None,
    };
    Some(match (name, max) {
        // Signed integers: 2's-complement range.
        ("BYTE", false)     => IrValue::ConstInt(0, IrType::U8),
        ("BYTE", true)      => IrValue::ConstInt(255, IrType::U8),
        ("SHORTINT", false) => IrValue::ConstInt(i16::MIN as i128, IrType::I16),
        ("SHORTINT", true)  => IrValue::ConstInt(i16::MAX as i128, IrType::I16),
        ("INTSHORT", false) => IrValue::ConstInt(i32::MIN as i128, IrType::I32),
        ("INTSHORT", true)  => IrValue::ConstInt(i32::MAX as i128, IrType::I32),
        ("INTEGER", false)  => IrValue::ConstInt(i64::MIN as i128, IrType::I64),
        ("INTEGER", true)   => IrValue::ConstInt(i64::MAX as i128, IrType::I64),
        ("LONGINT", false)  => IrValue::ConstInt(i64::MIN as i128, IrType::I64),
        ("LONGINT", true)   => IrValue::ConstInt(i64::MAX as i128, IrType::I64),
        // Character types: ordinal range. SHORTCHAR is 8-bit (0..255).
        // CHAR is a 32-bit Unicode scalar (0..10FFFF inclusive).
        ("SHORTCHAR", false) => IrValue::ConstInt(0, IrType::ShortChar),
        ("SHORTCHAR", true)  => IrValue::ConstInt(0xFF, IrType::ShortChar),
        ("CHAR", false)      => IrValue::ConstInt(0, IrType::Char),
        ("CHAR", true)       => IrValue::ConstInt(0x10_FFFF, IrType::Char),
        // SET range: returned as INTEGER per CP spec.
        ("SET", false) => IrValue::ConstInt(0, IrType::I64),
        ("SET", true)  => IrValue::ConstInt(31, IrType::I64),
        // Real types — CP defines MIN(REAL) as the smallest positive value
        // (the IEEE-754 normalized minimum) and MAX(REAL) as the largest finite
        // value. SHORTREAL = f32, REAL = f64.
        ("SHORTREAL", false) => IrValue::ConstReal(f32::MIN_POSITIVE as f64, IrType::F32),
        ("SHORTREAL", true)  => IrValue::ConstReal(f32::MAX as f64, IrType::F32),
        ("REAL", false)      => IrValue::ConstReal(f64::MIN_POSITIVE, IrType::F64),
        ("REAL", true)       => IrValue::ConstReal(f64::MAX, IrType::F64),
        _ => return None,
    })
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
        // String = null-terminated array of CHAR (32-bit Unicode scalar values).
        // ShortString = null-terminated array of SHORTCHAR (8-bit bytes).
        BuiltinType::String => IrType::Ptr(Box::new(IrType::Char)),
        BuiltinType::ShortString => IrType::Ptr(Box::new(IrType::ShortChar)),
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

    fn record_diagnostic(&mut self, message: impl Into<String>) {
        self.proc.diagnostics.push(LoweringDiagnostic {
            message: message.into(),
        });
    }

    /// If `type_name` is a named pointer alias, return the concrete IR pointer type.
    ///
    /// E.g. `DataPtr = POINTER TO Data` → `Some(IrType::Ptr(Named("Data")))`.
    fn resolve_named_as_ptr_ir_type(&self, type_name: &str) -> Option<IrType> {
        // Cross-module type alias `"Module.Type"` — load the imported
        // module's sema and resolve there. Required so field-access on a
        // local var typed as `Mod.Foo` (e.g. `sl: HostFiles.StdLocator`)
        // sees Foo as a pointer alias and inserts the necessary Load.
        if let Some((module, name)) = type_name.split_once('.') {
            let sema = find_imported_module_sema(module)?;
            let local_type_names: std::collections::HashSet<String> = sema
                .symbols
                .iter()
                .filter(|sym| sym.kind == SymbolKind::Type)
                .map(|sym| sym.name.clone())
                .collect();
            let mut ty = sema
                .symbols
                .iter()
                .find(|sym| sym.kind == SymbolKind::Type && sym.name == name)
                .and_then(|sym| sym.declared_type.as_ref())?
                .clone();
            // Qualify any internal `Named { module: None, name: T }` refs
            // (e.g. the pointee `StdLocatorDesc` inside the `Pointer`)
            // with the importing module name so that downstream IR
            // lookups know to route to the imported module's symbols.
            qualify_local_named_refs_in_sem_type(&mut ty, module, &local_type_names);
            return match ty {
                SemanticType::Pointer { .. } => Some(map_semantic_type(&ty)),
                _ => None,
            };
        }

        let ty = self
            .symbols
            .iter()
            .rev()
            .chain(self.module_symbols.iter().rev())
            .find(|sym| sym.kind == SymbolKind::Type && sym.name == type_name)
            .and_then(|sym| sym.declared_type.as_ref())?;
        match ty {
            SemanticType::Pointer { target, untagged } => {
                // Special case: if the target is a Named alias that
                // resolves to an open-array (`Bag = POINTER TO DigitArr;
                // DigitArr = ARRAY OF SHORTINT`), unwrap one level so
                // `map_semantic_type`'s open-array-target shortcut fires
                // and produces `Ptr(SHORTINT)` instead of
                // `Ptr(Named("DigitArr"))`. For all other targets
                // (records, etc.) leave the Named in place so
                // downstream IR can still find the named struct type.
                let resolved_target = self.resolve_named_sem_type(target);
                let target_for_map = match &resolved_target {
                    SemanticType::Array { lengths, .. } if lengths.is_empty() => resolved_target,
                    _ => target.as_ref().clone(),
                };
                let synthesised = SemanticType::Pointer {
                    target: Box::new(target_for_map),
                    untagged: *untagged,
                };
                Some(map_semantic_type(&synthesised))
            }
            SemanticType::Named { module: None, name, .. } if name != type_name => {
                self.resolve_named_as_ptr_ir_type(name)
            }
            _ => None,
        }
    }

    /// Walk Named-aliased semantic types one level (local symbols only)
    /// so a `Named { name: T }` referring to a type definition resolves
    /// to that type's declared form. Used by IR-type derivation to flatten
    /// `Bag = POINTER TO DigitArr; DigitArr = ARRAY OF SHORTINT` into
    /// `POINTER TO ARRAY OF SHORTINT` for the special-case mapping in
    /// `map_semantic_type`.
    fn resolve_named_sem_type(&self, ty: &SemanticType) -> SemanticType {
        match ty {
            SemanticType::Named { module: None, name, .. } => {
                self.symbols
                    .iter()
                    .rev()
                    .chain(self.module_symbols.iter().rev())
                    .find(|sym| sym.kind == SymbolKind::Type && sym.name == *name)
                    .and_then(|sym| sym.declared_type.clone())
                    .unwrap_or_else(|| ty.clone())
            }
            _ => ty.clone(),
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
    fn base_symbol_ir_type(&mut self, qual: &QualIdent) -> Option<IrType> {
        // A module-qualified base (e.g. `WinFrame.BufPersistent`) refers to an
        // exported symbol of an imported module — never a local. Look it up in
        // the imported module's analysed symbol table and bypass the local
        // chain entirely. Without this, cross-module CONST/VAR references fall
        // through to the `Opaque("unresolved")` fallback at the call site,
        // which lowers to `ptr` and silently corrupts the call ABI.
        if let Some(module_name) = qual.module.as_deref() {
            let sema = load_cached_import(module_name, &mut self.import_cache)?;
            // Variable declarations in the imported module record their type
            // unqualified (e.g. `theDir: StdDir` inside `HostFiles.cp`
            // stores `Named{module: None, name: "StdDir"}`). When we read
            // that type from outside the module, we must qualify any
            // module-local Named refs so that downstream IR lookups
            // (`flatten_fields_for_ir_type`, `lower_bound_proc_call_expr`)
            // can route to the right module's symbols.
            let local_type_names: std::collections::HashSet<String> = sema
                .symbols
                .iter()
                .filter(|sym| sym.kind == SymbolKind::Type)
                .map(|sym| sym.name.clone())
                .collect();
            return sema
                .symbols
                .iter()
                .find(|symbol| symbol.name == qual.name)
                .and_then(|symbol| symbol.declared_type.as_ref())
                .cloned()
                .map(|mut ty| {
                    qualify_local_named_refs_in_sem_type(
                        &mut ty,
                        module_name,
                        &local_type_names,
                    );
                    map_semantic_type(&ty)
                });
        }

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
                    let result = if let Some((module, name)) = n.split_once('.') {
                        self.flatten_imported_record_fields(module, name)
                    } else {
                        // For local named types, use flatten_sem_type_fields which resolves
                        // Named base types (e.g. `Bird RECORD (Animal)` where Animal is Named).
                        let sem_ty = self
                            .symbols
                            .iter()
                            .rev()
                            .chain(self.module_symbols.iter())
                            .find(|sym| sym.kind == SymbolKind::Type && sym.name == n.as_str())
                            .and_then(|s| s.declared_type.as_ref());
                        Self::flatten_sem_type_fields(sem_ty, self.module_symbols)
                    };
                    if std::env::var("NEWCP_IR_DEBUG").is_ok() {
                        eprintln!(
                            "[ir] flatten_fields_for {n}: {} fields = {:?}",
                            result.len(),
                            result.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
                        );
                    }
                    return result;
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
                            // Cross-module base. Use the same path-search
                            // strategy as `load_cached_import` (sibling-of-
                            // importing-source, walk-up-to-Mod/, then
                            // cwd-relative `Mod/`) so fixtures in
                            // `Mod/Tests/` can find imports next to them
                            // or in the parent `Mod/` directory.
                            if let Some(base_sema) = find_imported_module_sema(m) {
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
            // Pointer alias (`Foo = POINTER TO FooDesc`): flatten the
            // pointed-to type. Required so `flatten_fields_for_ir_type`
            // resolves field accesses on a pointer-aliased type to the
            // record's fields rather than returning an empty list.
            SemanticType::Pointer { target, .. } => {
                Self::flatten_sem_type_fields(Some(target.as_ref()), module_symbols)
            }
            // Cross-module Named: load the imported module's symbols and
            // recurse. Without this, a pointer alias `Foo.Bar` whose
            // pointee is `Foo.BarDesc` would stop at the Named here.
            SemanticType::Named { module: Some(m), name, .. } => {
                if let Some(base_sema) = find_imported_module_sema(m) {
                    let sym = base_sema
                        .symbols
                        .iter()
                        .find(|s| s.name == *name && s.kind == SymbolKind::Type);
                    return Self::flatten_sem_type_fields(
                        sym.and_then(|s| s.declared_type.as_ref()),
                        &base_sema.symbols,
                    );
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }
}

// == Expression lowering ==

impl<'m> LowerCtx<'m> {
    fn normalize_designator(&mut self, des: &Designator) -> Designator {
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
            Expr::Set { elements, .. } => {
                // Build the SET value by ORing in each element.
                // For a singleton element `e`, this is: acc | (1 << e).
                // For a range element `e1..e2`, we OR in all bits from e1 to e2 inclusive.
                // Start with an empty set (0) then fold.
                let mut acc: IrValue = IrValue::ConstInt(0, IrType::Set(32));
                for elem in elements {
                    // Evaluate the lower bound.
                    let start_val = self.lower_expr(&elem.start);
                    // Widen to i32 if needed (set ops are 32-bit).
                    let start_i32 = if start_val.ty() != IrType::Set(32) {
                        let t = self.fresh_temp();
                        self.push(Instr::Cast { dst: t, value: start_val, to_ty: IrType::Set(32) });
                        IrValue::Temp(t, IrType::Set(32))
                    } else {
                        start_val
                    };

                    let bit_val = if let Some(end_expr) = &elem.end {
                        // Range e1..e2: build mask with all bits from e1 to e2.
                        // mask = ((2 << e2) - 1) & ~((1 << e1) - 1)
                        // which is simpler as: ((1 << (e2 - e1 + 1)) - 1) << e1
                        // But to stay general (runtime values): use a loop or the bitmask formula.
                        // Simpler formula using i64 intermediate:
                        //   bit_range = ((1u64 << (e2 + 1)) - 1) ^ ((1u64 << e1) - 1)
                        // We emit: t_two = 2; (t_two << e2) - 1 = mask_high; (1 << e1) - 1 = mask_low; bit_range = mask_high XOR mask_low
                        let end_val = self.lower_expr(end_expr);
                        let end_i32 = if end_val.ty() != IrType::Set(32) {
                            let t = self.fresh_temp();
                            self.push(Instr::Cast { dst: t, value: end_val, to_ty: IrType::Set(32) });
                            IrValue::Temp(t, IrType::Set(32))
                        } else {
                            end_val
                        };
                        // two_shifted_end = 2 << e2  (= 1 << (e2+1))
                        let t_two = self.fresh_temp();
                        self.push(Instr::BinOp { dst: t_two, op: BinOp::Shl,
                            left: IrValue::ConstInt(2, IrType::Set(32)),
                            right: end_i32,
                            ty: IrType::Set(32) });
                        // mask_high = (2 << e2) - 1
                        let t_mh = self.fresh_temp();
                        self.push(Instr::BinOp { dst: t_mh, op: BinOp::Sub,
                            left: IrValue::Temp(t_two, IrType::Set(32)),
                            right: IrValue::ConstInt(1, IrType::Set(32)),
                            ty: IrType::Set(32) });
                        // one_shifted_start = 1 << e1
                        let t_oss = self.fresh_temp();
                        self.push(Instr::BinOp { dst: t_oss, op: BinOp::Shl,
                            left: IrValue::ConstInt(1, IrType::Set(32)),
                            right: start_i32,
                            ty: IrType::Set(32) });
                        // mask_low = (1 << e1) - 1
                        let t_ml = self.fresh_temp();
                        self.push(Instr::BinOp { dst: t_ml, op: BinOp::Sub,
                            left: IrValue::Temp(t_oss, IrType::Set(32)),
                            right: IrValue::ConstInt(1, IrType::Set(32)),
                            ty: IrType::Set(32) });
                        // bit_range = mask_high XOR mask_low (= (2<<e2)-1 minus (1<<e1)-1)
                        let t_range = self.fresh_temp();
                        self.push(Instr::BinOp { dst: t_range, op: BinOp::Xor,
                            left: IrValue::Temp(t_mh, IrType::Set(32)),
                            right: IrValue::Temp(t_ml, IrType::Set(32)),
                            ty: IrType::Set(32) });
                        IrValue::Temp(t_range, IrType::Set(32))
                    } else {
                        // Singleton: bit = 1 << e
                        let t_bit = self.fresh_temp();
                        self.push(Instr::BinOp { dst: t_bit, op: BinOp::Shl,
                            left: IrValue::ConstInt(1, IrType::Set(32)),
                            right: start_i32,
                            ty: IrType::Set(32) });
                        IrValue::Temp(t_bit, IrType::Set(32))
                    };

                    // acc = acc | bit_val
                    let t_or = self.fresh_temp();
                    let acc_prev = acc;
                    self.push(Instr::BinOp { dst: t_or, op: BinOp::Or,
                        left: acc_prev,
                        right: bit_val,
                        ty: IrType::Set(32) });
                    acc = IrValue::Temp(t_or, IrType::Set(32));
                }
                acc
            }
        }
    }

    fn lower_literal(&self, lit: &Literal) -> IrValue {
        match lit {
            Literal::Integer(s) => {
                let v: i128 = parse_cp_integer_literal(s);
                IrValue::ConstInt(v, IrType::I64)
            }
            Literal::Real(s) => {
                let v: f64 = s.parse().unwrap_or(0.0);
                IrValue::ConstReal(v, IrType::F64)
            }
            Literal::Character(s) => {
                if s.starts_with('"') || s.starts_with('\'') {
                    // Quoted single character: 'x' or "x" — CP CHAR (u32/i32).
                    let inner = &s[1..s.len()-1];
                    let c = inner.chars().next().unwrap_or('\0');
                    IrValue::ConstChar(c)
                } else {
                    // Hex character literal: NNX — ordinal is the hex value.
                    // ordinal <= 0xFF → SHORTCHAR (i8); > 0xFF → CHAR (i32/u32).
                    let hex = s.strip_suffix('X').unwrap_or(s);
                    let ordinal = i128::from_str_radix(hex, 16).unwrap_or(0);
                    if ordinal <= 0xFF {
                        IrValue::ConstInt(ordinal, IrType::ShortChar)
                    } else {
                        IrValue::ConstInt(ordinal, IrType::Char)
                    }
                }
            }
            Literal::String(s) => {
                let inner = s.trim_matches('"').trim_matches('\'');
                let mut chars = inner.chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    IrValue::ConstChar(c)
                } else {
                    IrValue::ConstStr(inner.to_string(), IrType::Char)
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
                let mut uv_args: Vec<IrValue> = Vec::with_capacity(upvalues.len());
                for (uv_name, uv_ty) in &upvalues {
                    uv_args.push(IrValue::GlobalRef(
                        uv_name.clone(),
                        IrType::Ref(Box::new(map_semantic_type(uv_ty))),
                    ));
                    // Open-array upvalues travel with their hidden length companion.
                    if is_open_array(uv_ty) {
                        let hidden = format!("{uv_name}{OPEN_ARRAY_LEN_SUFFIX}");
                        uv_args.push(IrValue::GlobalRef(
                            hidden,
                            IrType::Ref(Box::new(IrType::I64)),
                        ));
                    }
                }
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
                None => {
                    // Bare procedure call with no parentheses: `Log.Open` or `Flush`
                    // is valid CP for a parameterless procedure call.
                    let ret_ty = self.callee_return_type(&callee);
                    let args = upvalue_args;
                    if ret_ty == IrType::Void {
                        self.push(Instr::Call {
                            dst: None,
                            callee: callee.clone(),
                            args,
                            ret_ty,
                        });
                        return callee;
                    }
                    let t = self.fresh_temp();
                    self.push(Instr::Call {
                        dst: Some(t),
                        callee,
                        args,
                        ret_ty: ret_ty.clone(),
                    });
                    return IrValue::Temp(t, ret_ty);
                }
                _ => return callee,
            }
        }

        // Indirect procedure-variable call: `fn()` or `fn(a, b)` where `fn`
        // is a local variable declared as a named procedure-type alias.
        // `is_direct_callee` returned false because the symbol is a local var
        // (SymbolKind::LocalVar), not a Procedure.  We detect this case here
        // and emit a Load of the function pointer followed by an indirect Call.
        if let Some(Selector::Call(call_args)) = des.selectors.first() {
            if let Some(proc_sig) = self.base_proc_type(&base_name) {
                let call_args = call_args.clone();
                // Load the function pointer from the variable (strip the Call selector).
                let base_des = Designator {
                    span: des.span,
                    base: des.base.clone(),
                    selectors: Vec::new(),
                };
                let fn_addr = self.designator_addr(&base_des);
                let fn_ptr_t = self.fresh_temp();
                self.push(Instr::Load { dst: fn_ptr_t, addr: fn_addr, ty: final_ty.clone() });
                let fn_ptr_val = IrValue::Temp(fn_ptr_t, final_ty);

                let ret_ty = proc_sig.result_type.as_ref()
                    .map(|rt| map_semantic_type(rt.as_ref()))
                    .unwrap_or(IrType::Void);

                // Lower arguments using the known procedure signature.
                let param_modes: Vec<Option<ParamMode>> = proc_sig.parameters.iter()
                    .flat_map(|p| std::iter::repeat_n(p.mode, p.names.len()))
                    .collect();
                let param_tys: Vec<SemanticType> = proc_sig.parameters.iter()
                    .flat_map(|p| std::iter::repeat_n(p.ty.clone(), p.names.len()))
                    .collect();
                let call_args_lowered: Vec<IrValue> = call_args.iter().enumerate().map(|(i, arg)| {
                    let mode = param_modes.get(i).copied().flatten();
                    if matches!(mode, Some(ParamMode::Var) | Some(ParamMode::Out)) {
                        match arg {
                            Expr::Designator(des) => self.designator_addr(des),
                            _ => self.lower_expr(arg),
                        }
                    } else {
                        let is_open_arr = param_tys.get(i).map(|t| {
                            matches!(t, SemanticType::Array { lengths, .. } if lengths.is_empty())
                        }).unwrap_or(false);
                        if is_open_arr {
                            if let Expr::Designator(des) = arg {
                                if matches!(self.designator_ir_type(des), Some(IrType::Array { .. })) {
                                    return self.designator_addr(des);
                                }
                            }
                        }
                        self.lower_expr(arg)
                    }
                }).collect();

                if ret_ty == IrType::Void {
                    self.push(Instr::Call {
                        dst: None,
                        callee: fn_ptr_val.clone(),
                        args: call_args_lowered,
                        ret_ty,
                    });
                    return fn_ptr_val;
                }
                let call_t = self.fresh_temp();
                self.push(Instr::Call {
                    dst: Some(call_t),
                    callee: fn_ptr_val,
                    args: call_args_lowered,
                    ret_ty: ret_ty.clone(),
                });
                return IrValue::Temp(call_t, ret_ty);
            }
        }

        // Procedure value used as a first-class value (e.g. assigned to a proc-type variable).
        // When there are no selectors and the base name refers to a procedure (not a var),
        // return the procedure reference directly without emitting a Load.
        if des.selectors.is_empty() {
            let is_procedure = module_opt.is_none() && self.symbols.iter().rev()
                .chain(self.module_symbols.iter().rev())
                .find(|s| s.name == base_name)
                .map(|s| matches!(s.kind, SymbolKind::Procedure))
                .unwrap_or(false);
            if is_procedure {
                return IrValue::GlobalRef(base_name, final_ty);
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
        // The parser greedily packs `b.Method` as `QualIdent { module: Some("b"),
        // name: "Method" }`. `normalize_designator` turns it back into base="b"
        // + Field("Method") when "b" is a local rather than a module — match the
        // multi-selector shape so the rest of this function can handle both.
        let des_normalized = self.normalize_designator(des);
        let des = &des_normalized;

        // Pattern: selectors end with [.., Field(method_name), Call(args)]
        // or [.., Field(method_name), AmbiguousParen(single_qualident)].
        // The parser emits `AmbiguousParen` when the parenthesised content
        // is a single qualident that could be either a type-guard target
        // or a one-argument call; for method dispatch it's always a call.
        let selectors = &des.selectors;
        let n = selectors.len();
        if n < 2 {
            return None;
        }
        let Selector::Field(method_name) = &selectors[n - 2] else {
            return None;
        };
        // Materialise the call arguments. For `Selector::AmbiguousParen`
        // we synthesise a single-element Vec<Expr> wrapping the qualident.
        let synthetic_args: Vec<Expr>;
        let call_args: &Vec<Expr> = match &selectors[n - 1] {
            Selector::Call(args) => args,
            Selector::AmbiguousParen(qual) => {
                synthetic_args = vec![Expr::Designator(Designator {
                    span: qual.span,
                    base: qual.clone(),
                    selectors: Vec::new(),
                })];
                &synthetic_args
            }
            _ => return None,
        };

        // Resolve the receiver designator (everything before the last two selectors).
        // We need to know the RECORD type of the receiver so we can look up the slot.
        let prefix_des = Designator {
            span: des.span,
            base: des.base.clone(),
            selectors: selectors[..n - 2].to_vec(),
        };
        let prefix_ty = self.designator_ir_type(&prefix_des);
        if std::env::var("NEWCP_IR_DEBUG").is_ok() {
            eprintln!(
                "[ir] lower_bound_proc_call_expr method={method_name} \
                 prefix_des.base={:?}.{:?} prefix_ty={prefix_ty:?}",
                des.base.module, des.base.name,
            );
        }
        let prefix_ty = prefix_ty?;

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

        // The receiver's type may be cross-module (e.g. `f: Files.File`).
        // For local types the existing `module_symbols` suffice; for
        // imported types we load the imported module's sema once and
        // pass its symbols down. We hold the imported `SemanticModule`
        // for the rest of the function so the symbol slice it provides
        // remains valid while we build the IR for this call.
        let (type_local_name, imported_sema_holder): (&str, Option<SemanticModule>) =
            match type_qualified.split_once('.') {
                Some((module_name, local_name)) => (local_name, find_imported_module_sema(module_name)),
                None => (type_qualified, None),
            };
        let lookup_symbols: &[SemanticSymbol] = imported_sema_holder
            .as_ref()
            .map(|s| s.symbols.as_slice())
            .unwrap_or(self.module_symbols);

        // Resolve a pointer alias (`Foo = POINTER TO FooDesc`) down to the
        // underlying record type name. Methods are declared on the record,
        // not the pointer alias, and the vtable / sema lookups below use
        // the record name.
        let type_record_name: &str = lookup_symbols
            .iter()
            .find(|s| matches!(s.kind, SymbolKind::Type) && s.name == type_local_name)
            .and_then(|s| s.declared_type.as_ref())
            .and_then(|t| match t {
                SemanticType::Pointer { target, .. } => match target.as_ref() {
                    SemanticType::Named { name, module: None, .. } => Some(name.as_str()),
                    _ => None,
                },
                _ => None,
            })
            .unwrap_or(type_local_name);

        // Check that method_name is actually a METHOD (not a data field) of this type.
        // Use the (possibly imported) sema symbol table to look at Record.methods.
        if std::env::var("NEWCP_IR_DEBUG").is_ok() {
            eprintln!(
                "[ir] dispatch lookup: prefix_ty={prefix_ty:?} type_qualified={type_qualified} \
                 type_local={type_local_name} type_record={type_record_name} method={method_name} \
                 imported_sema={}",
                imported_sema_holder.is_some()
            );
        }
        let slot = method_slot_in_vtable(type_record_name, method_name, lookup_symbols)?;

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

        // Look up the method's full signature in the receiver's module
        // symbols. We need both the parameter modes/types (for arg
        // lowering) and the result type (for the call's IR result).
        let method_sig: Option<newcp_sema::ProcedureType> = lookup_symbols
            .iter()
            .find(|s| {
                s.kind == SymbolKind::Procedure
                    && s.name == *method_name
                    && s.declared_type.as_ref().and_then(|t| {
                        if let SemanticType::Procedure(pt) = t { pt.receiver.as_deref() } else { None }
                    }).and_then(|r| match r {
                        SemanticType::Named { name, .. } => Some(name.as_str()),
                        _ => None,
                    }) == Some(type_record_name)
            })
            .and_then(|s| match &s.declared_type {
                Some(SemanticType::Procedure(pt)) => Some(pt.clone()),
                _ => None,
            });

        // Lower the call args using the shared signature-driven helper so
        // open-array fat-pointer ABI, VAR-mode address passing, fixed-
        // array-by-reference, and SHORTCHAR widening all work the same
        // for method calls as they do for direct procedure calls.
        // (Receiver is carried in MethodCall::descriptor, not in args.)
        let (expected_modes, expected_types) =
            flatten_param_modes_and_types(method_sig.as_ref());
        let lowered_args =
            self.lower_args_with_signature(call_args, &expected_modes, &expected_types);

        let ret_ty = method_sig
            .as_ref()
            .map(|pt| pt.result_type.as_ref().map(|t| map_semantic_type(t)).unwrap_or(IrType::Void))
            .unwrap_or(IrType::Void);

        if ret_ty == IrType::Void {
            // Void methods are typically called as statements (`b.Set(42);`).
            // Emit the dispatch and return a placeholder; the statement path
            // discards it. Sema rejects void-returning calls in expression
            // position, so reaching here from an expression is well-formed
            // only when the caller intends to ignore the value.
            self.push(Instr::MethodCall {
                dst: None,
                descriptor: receiver_ptr,
                slot,
                args: lowered_args,
                ret_ty: IrType::Void,
            });
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
        &mut self,
        module_name: Option<&str>,
        base_name: &str,
        ty: &IrType,
    ) -> Option<IrValue> {
        // Resolve to either a local symbol or an exported symbol of an imported
        // module. The two arms must agree on the const-value lowering so that
        // `WinFrame.BufPersistent` produces an `IrValue::ConstInt(1, i32)` exactly
        // as a local `BufPersistent` would. Without the imported arm the call
        // site would emit `load WinFrame.BufPersistent` with `Opaque("unresolved")`
        // type, lower to `ptr` in LLVM, and prepend a phantom argument that
        // shifts every subsequent argument across the call ABI.
        let const_value = if let Some(module) = module_name {
            let sema = load_cached_import(module, &mut self.import_cache)?;
            let symbol = sema
                .symbols
                .iter()
                .find(|symbol| symbol.name == base_name)?;
            symbol.const_value.clone()?
        } else {
            let symbol = self
                .symbols
                .iter()
                .rev()
                .find(|symbol| symbol.name == base_name)?;
            symbol.const_value.clone()?
        };
        Some(match const_value {
            ConstValue::Integer(value) => IrValue::ConstInt(value, ty.clone()),
            ConstValue::Real(value) => IrValue::ConstReal(value, ty.clone()),
            ConstValue::String(value) => IrValue::ConstStr(value, IrType::Char),
            ConstValue::Char(value) => IrValue::ConstChar(value),
            ConstValue::Boolean(value) => IrValue::ConstBool(value),
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
                                // Ref(Ref(...)) — VAR param. The LLVM `ref_param_slots`
                                // mechanism in `resolve_pointer` already loads one level
                                // of indirection, so we treat `addr` as if it had only one
                                // outer Ref and pattern-match the inner type accordingly.
                                IrType::Ref(inner2) => match inner2.as_ref() {
                                    IrType::Array { element, len } => {
                                        (addr.clone(), *element.clone(), Some(*len))
                                    }
                                    IrType::Ptr(elem) | IrType::UntaggedPtr(elem) => {
                                        // VAR open-array: addr resolves to the array base
                                        // pointer directly via ref_param_slots; no extra Load.
                                        (addr.clone(), *elem.clone(), None)
                                    }
                                    // VAR/OUT param of a pointer-to-array alias
                                    // (e.g. `OUT quo: Integer` where
                                    // `Integer = POINTER TO ARRAY OF Digit`).
                                    // ref_param_slots has already loaded one
                                    // indirection so addr behaves like
                                    // `Ref(Named(...))`. Resolve the alias to
                                    // get the underlying element type, then
                                    // load the pointer and index.
                                    IrType::Named(n) => match self.resolve_named_as_ptr_ir_type(n) {
                                        Some(IrType::Ptr(elem)) => {
                                            let loaded_ptr_ty = IrType::Ptr(elem.clone());
                                            let t = self.fresh_temp();
                                            self.push(Instr::Load {
                                                dst: t,
                                                addr: addr.clone(),
                                                ty: loaded_ptr_ty.clone(),
                                            });
                                            (IrValue::Temp(t, loaded_ptr_ty), *elem, None)
                                        }
                                        Some(IrType::UntaggedPtr(elem)) => {
                                            let loaded_ptr_ty = IrType::UntaggedPtr(elem.clone());
                                            let t = self.fresh_temp();
                                            self.push(Instr::Load {
                                                dst: t,
                                                addr: addr.clone(),
                                                ty: loaded_ptr_ty.clone(),
                                            });
                                            (IrValue::Temp(t, loaded_ptr_ty), *elem, None)
                                        }
                                        _ => (addr.clone(), IrType::Opaque("array-elem".to_string()), None),
                                    },
                                    _ => (addr.clone(), IrType::Opaque("array-elem".to_string()), None),
                                },
                                // Ref(Named) — resolve the named type's underlying form
                                // and dispatch the same way as the direct Ref(Ptr|Array|...) cases.
                                // Required for `Bag = POINTER TO ARRAY OF SHORTINT` →
                                // need to load the pointer, then index by element.
                                IrType::Named(n) => {
                                    match self.resolve_named_as_ptr_ir_type(n) {
                                        Some(IrType::Ptr(elem)) => {
                                            let loaded_ptr_ty = IrType::Ptr(elem.clone());
                                            let t = self.fresh_temp();
                                            self.push(Instr::Load {
                                                dst: t,
                                                addr: addr.clone(),
                                                ty: loaded_ptr_ty.clone(),
                                            });
                                            (IrValue::Temp(t, loaded_ptr_ty), *elem, None)
                                        }
                                        Some(IrType::UntaggedPtr(elem)) => {
                                            let loaded_ptr_ty = IrType::UntaggedPtr(elem.clone());
                                            let t = self.fresh_temp();
                                            self.push(Instr::Load {
                                                dst: t,
                                                addr: addr.clone(),
                                                ty: loaded_ptr_ty.clone(),
                                            });
                                            (IrValue::Temp(t, loaded_ptr_ty), *elem, None)
                                        }
                                        _ => (addr.clone(), IrType::Opaque("array-elem".to_string()), None),
                                    }
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

    /// Resolve the variable `name` to a `ProcedureType` if its declared type is
    /// a named procedure-type alias (e.g. `fn: NullaryIntProc` where
    /// `TYPE NullaryIntProc = PROCEDURE(): INTEGER`).
    fn base_proc_type(&self, name: &str) -> Option<newcp_sema::ProcedureType> {
        // 1. Find the local / module symbol and its declared semantic type.
        let sym_ty = self.symbols.iter().rev()
            .chain(self.module_symbols.iter().rev())
            .find(|s| s.name == name)?
            .declared_type.as_ref()?;
        // 2. If it is directly a procedure type, return it immediately.
        if let SemanticType::Procedure(sig) = sym_ty {
            return Some(sig.clone());
        }
        // 3. Otherwise it must be a Named alias — extract the alias name.
        let type_name = match sym_ty {
            SemanticType::Named { name, module: None, .. } => name.as_str(),
            _ => return None,
        };
        // 4. Look up the type definition and return the procedure signature.
        self.symbols.iter().rev()
            .chain(self.module_symbols.iter().rev())
            .find(|s| s.kind == SymbolKind::Type && s.name == type_name)
            .and_then(|s| s.declared_type.as_ref())
            .and_then(|ty| match ty {
                SemanticType::Procedure(sig) => Some(sig.clone()),
                _ => None,
            })
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
        // Param/result types in the imported procedure carry
        // `Named { module: None, name: T }` for any T defined locally
        // in that module — but when consulted from outside, those refs
        // need to be qualified with the source module so downstream IR
        // routing (record-by-reference detection, fat-pointer length
        // resolution, …) routes to the right module's symbols.
        let local_type_names: std::collections::HashSet<String> = sema
            .symbols
            .iter()
            .filter(|sym| sym.kind == SymbolKind::Type)
            .map(|sym| sym.name.clone())
            .collect();
        let mut signature = sema
            .procedures
            .iter()
            .find(|proc| proc.name == name && proc.exported)
            .map(|proc| proc.signature.clone())?;
        for param in &mut signature.parameters {
            qualify_local_named_refs_in_sem_type(&mut param.ty, module, &local_type_names);
        }
        if let Some(result) = signature.result_type.as_mut() {
            qualify_local_named_refs_in_sem_type(result, module, &local_type_names);
        }
        Some(signature)
    }

    /// True when `name` is a parameter whose IR type is `Ref(_)` (i.e.
    /// declared with VAR / OUT / IN mode). Used to disambiguate forwarding
    /// open-array arguments: a Ref param's slot is read via the LLVM
    /// ref_param_slots indirection (one deref), whereas a value-mode param's
    /// slot just holds the data pointer (no indirection).
    fn is_ref_typed_param(&self, name: &str) -> bool {
        self.proc
            .params
            .iter()
            .find(|(n, _)| n == name)
            .is_some_and(|(_, ty)| matches!(ty, IrType::Ref(_)))
    }

    /// Compute the length (element count) to pass for an open-array argument.
    ///   - Designator → fixed-size array        : statically known length
    ///   - Designator → forwarded open-array    : load `<src>$len`
    ///   - String literal (ConstStr)            : char count + 1 (NUL terminator)
    ///   - Single char promoted to ConstStr     : 2
    ///   - Anything else                        : 0 (defensive; sema should reject)
    fn compute_open_array_len(&mut self, arg: &Expr, lowered: &IrValue) -> IrValue {
        if let Expr::Designator(des) = arg {
            // Forwarding case: bare param name with a hidden `<name>$len` companion.
            if des.selectors.is_empty() && des.base.module.is_none() {
                let hidden = format!("{}{OPEN_ARRAY_LEN_SUFFIX}", des.base.name);
                if self.proc.params.iter().any(|(n, _)| n == &hidden) {
                    let addr = IrValue::GlobalRef(hidden, IrType::Ref(Box::new(IrType::I64)));
                    let t = self.fresh_temp();
                    self.push(Instr::Load { dst: t, addr, ty: IrType::I64 });
                    return IrValue::Temp(t, IrType::I64);
                }
            }
            // Fixed-size array source: walk the IR type to recover the length.
            // The type may be a `Named` type alias (local OR cross-module);
            // resolve through aliases until we either reach a concrete
            // Array or give up.
            if let Some(ty) = self.designator_ir_type(des) {
                if let Some(len) = self.resolve_fixed_array_len(&ty) {
                    return IrValue::ConstInt(len as i128, IrType::I64);
                }
            }
        }
        // String literal capacity = char count + 1 (for the trailing NUL).
        if let IrValue::ConstStr(s, _) = lowered {
            return IrValue::ConstInt((s.chars().count() as i128) + 1, IrType::I64);
        }
        IrValue::ConstInt(0, IrType::I64)
    }

    /// If `ty` is (or resolves through aliases to) `IrType::Array { len, .. }`,
    /// return that length. Handles both local Named aliases and cross-module
    /// `Mod.Type` aliases (e.g. `Files.Name` -> `[256 x char]`).
    fn resolve_fixed_array_len(&mut self, ty: &IrType) -> Option<u64> {
        match ty {
            IrType::Array { len, .. } => Some(*len as u64),
            IrType::Named(n) => {
                // Cross-module Named: `"Module.Type"`.
                if let Some((module, name)) = n.split_once('.') {
                    let sema = load_cached_import(module, &mut self.import_cache)?;
                    let sym = sema
                        .symbols
                        .iter()
                        .find(|s| s.kind == SymbolKind::Type && s.name == name)?;
                    let resolved = map_semantic_type(sym.declared_type.as_ref()?);
                    return self.resolve_fixed_array_len(&resolved);
                }
                // Local Named: walk module symbols.
                let sym = self
                    .symbols
                    .iter()
                    .rev()
                    .chain(self.module_symbols.iter())
                    .find(|s| s.kind == SymbolKind::Type && s.name == n.as_str())?;
                let resolved = map_semantic_type(sym.declared_type.as_ref()?);
                self.resolve_fixed_array_len(&resolved)
            }
            IrType::Ref(inner) => self.resolve_fixed_array_len(inner),
            _ => None,
        }
    }

    /// True if `ty` is (or resolves through Named aliases to) a Record.
    /// Pointer-aliased types do NOT count — they're scalars at the ABI
    /// level. Used by call arg lowering to decide whether a value-mode
    /// or IN-mode argument needs to be passed by address.
    fn semantic_resolves_to_record(&mut self, ty: &SemanticType) -> bool {
        match ty {
            SemanticType::Record { .. } => true,
            SemanticType::Pointer { .. } => false,
            SemanticType::Named { module: Some(m), name, .. } => {
                find_imported_module_sema(m)
                    .and_then(|sema| {
                        sema.symbols
                            .iter()
                            .find(|s| s.kind == SymbolKind::Type && s.name == *name)
                            .and_then(|s| s.declared_type.clone())
                    })
                    .map(|inner| self.semantic_resolves_to_record(&inner))
                    .unwrap_or(false)
            }
            SemanticType::Named { module: None, name, .. } => {
                let resolved = self
                    .symbols
                    .iter()
                    .rev()
                    .chain(self.module_symbols.iter())
                    .find(|s| s.kind == SymbolKind::Type && s.name == *name)
                    .and_then(|s| s.declared_type.clone());
                match resolved {
                    Some(inner) => self.semantic_resolves_to_record(&inner),
                    None => false,
                }
            }
            _ => false,
        }
    }

    fn lower_call_args(&mut self, callee: &IrValue, args: &[Expr]) -> Vec<IrValue> {
        let proc_ty = self.callee_procedure_type(callee);
        let (expected_modes, expected_types) = flatten_param_modes_and_types(proc_ty.as_ref());
        self.lower_args_with_signature(args, &expected_modes, &expected_types)
    }

    /// Shared arg-lowering used by both direct procedure calls and method
    /// calls. The caller supplies the per-position `mode` (CP `VAR`/`OUT`/
    /// `IN`/value) and `expected_type` so this works for any signature
    /// source (local procedure, imported procedure, or method on an
    /// imported record).
    fn lower_args_with_signature(
        &mut self,
        args: &[Expr],
        expected_modes: &[Option<ParamMode>],
        expected_types: &[SemanticType],
    ) -> Vec<IrValue> {
        let mut out: Vec<IrValue> = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            let mode = expected_modes.get(index).copied().flatten();
            let is_open_array_param = expected_types.get(index).map(is_open_array).unwrap_or(false);

            // Compute the value (pointer) IR.
            let value: IrValue;
            if matches!(mode, Some(ParamMode::Var) | Some(ParamMode::Out)) {
                // VAR/OUT: always pass address
                value = match arg {
                    Expr::Designator(des) => self.designator_addr(des),
                    _ => self.lower_expr(arg),
                };
            } else if is_open_array_param {
                // IN or value open-array param.
                if let Expr::Designator(des) = arg {
                    let des_ty = self.designator_ir_type(des);
                    let resolves_to_array = des_ty
                        .as_ref()
                        .and_then(|ty| self.resolve_fixed_array_len(ty))
                        .is_some();
                    if matches!(des_ty, Some(IrType::Array { .. })) || resolves_to_array {
                        // Fixed-size array source (possibly via a Named alias
                        // — local or cross-module): pass its base address.
                        value = self.designator_addr(des);
                    } else {
                        // Forwarding another open-array param. Two sub-cases:
                        //
                        // - source is a Ref-typed param (IN/VAR/OUT mode):
                        //   `designator_addr` yields `Ref(Ref(Ptr(T)))`; the
                        //   LLVM `call_args` path then calls `resolve_pointer`
                        //   which dereferences the ref_param_slot once and
                        //   gives us the data pointer. Using `lower_expr` here
                        //   would emit an extra `Load` that *re-dereferences*
                        //   the data pointer and reads garbage.
                        //
                        // - source is a value-mode param (no annotation):
                        //   the slot stores the data pointer directly with no
                        //   ref_param_slot indirection. `lower_expr` loads the
                        //   slot exactly once → data pointer. `designator_addr`
                        //   would pass the slot address (a ptr-to-ptr).
                        let is_ref_param = self.is_ref_typed_param(&des.base.name);
                        value = if is_ref_param {
                            self.designator_addr(des)
                        } else {
                            self.lower_expr(arg)
                        };
                    }
                } else {
                    // String literal etc.
                    let expects_shortchar = matches!(
                        expected_types.get(index),
                        Some(SemanticType::Array { element_type, .. })
                            if matches!(element_type.as_ref(), SemanticType::Builtin(BuiltinType::ShortChar))
                    );
                    let mut v = self.lower_expr(arg);
                    if expects_shortchar {
                        if let IrValue::ConstStr(_, elem_ty) = &mut v {
                            *elem_ty = IrType::ShortChar;
                        }
                        if let IrValue::ConstChar(c) = v {
                            let mut s = String::with_capacity(1);
                            s.push(c);
                            v = IrValue::ConstStr(s, IrType::ShortChar);
                        }
                    }
                    value = v;
                }
            } else if let Expr::Designator(des) = arg {
                // Fixed-size array passed to a non-open-array param (e.g. ARRAY 4 OF CHAR
                // expected type — still passed by reference per CP rules).
                // Also: a record-typed value passed where a record is expected
                // (IN d: Date / value-mode d: Date) — CP records are passed
                // by reference at the ABI level. Without this, the caller
                // emits `load d` (a struct value) and the callee tries to
                // GEP on the value, which LLVM rejects.
                let des_ty = self.designator_ir_type(des);
                let is_record_value = expected_types
                    .get(index)
                    .map(|ty| self.semantic_resolves_to_record(ty))
                    .unwrap_or(false);
                if matches!(des_ty, Some(IrType::Array { .. })) || is_record_value {
                    value = self.designator_addr(des);
                } else {
                    value = self.lower_expr(arg);
                }
            } else {
                value = self.lower_expr(arg);
            }

            // Widen SHORTCHAR -> CHAR when the declared param is CHAR.
            // `41X` and similar hex literals are typed as SHORTCHAR but a CHAR
            // formal expects a 32-bit value at the ABI level.
            let expects_char = matches!(expected_types.get(index), Some(SemanticType::Builtin(BuiltinType::Char)));
            let final_value = if expects_char && value.ty() == IrType::ShortChar {
                let t = self.fresh_temp();
                self.push(Instr::Cast { dst: t, value: value.clone(), to_ty: IrType::Char });
                IrValue::Temp(t, IrType::Char)
            } else {
                value
            };

            out.push(final_value.clone());

            // Open-array params travel with a hidden `<name>$len: I64` length arg
            // (decomposed fat-pointer ABI; mirrors `lower_procedure`).
            if is_open_array_param {
                let len = self.compute_open_array_len(arg, &final_value);
                out.push(len);
            }
        }
        out
    }

    fn lower_unary(&mut self, op: UnaryOp, expr: &Expr) -> IrValue {
        let operand = self.lower_expr(expr);
        let ty = operand.ty();
        // SET monadic minus = complement: -s = s XOR all-ones  (§8.2.3)
        if op == UnaryOp::Minus && ty == IrType::Set(32) {
            let t = self.fresh_temp();
            self.push(Instr::BinOp {
                dst: t,
                op: BinOp::Xor,
                left: operand,
                right: IrValue::ConstInt(-1, IrType::Set(32)),
                ty: IrType::Set(32),
            });
            return IrValue::Temp(t, IrType::Set(32));
        }
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
        let mut lv = self.lower_expr(left);
        let mut rv = self.lower_expr(right);

        // Integer -> float promotion for mixed-type expressions. Component
        // Pascal allows integer operands to participate in REAL/SHORTREAL
        // arithmetic; the integer is silently converted (`sitofp`) to the
        // matching float width. Without this, `realvar * (ORD(ch) - ORD('0'))`
        // crashes the LLVM backend with a type mismatch.
        let (lf, rf) = (is_float_ir(&lv.ty()), is_float_ir(&rv.ty()));
        let (li, ri) = (is_int_ir(&lv.ty()), is_int_ir(&rv.ty()));
        if lf && ri {
            let to = lv.ty();
            let t = self.fresh_temp();
            self.push(Instr::Cast { dst: t, value: rv, to_ty: to.clone() });
            rv = IrValue::Temp(t, to);
        } else if rf && li {
            let to = rv.ty();
            let t = self.fresh_temp();
            self.push(Instr::Cast { dst: t, value: lv, to_ty: to.clone() });
            lv = IrValue::Temp(t, to);
        }

        let ty = lv.ty();

        // SET arithmetic operators (8.2.3 of CP spec) use bitwise operations.
        // +  union          → OR
        // -  difference     → A AND (NOT B)  i.e. A AND (B XOR -1)
        // *  intersection   → AND
        // /  sym. diff.     → XOR
        if ty == IrType::Set(32) {
            return match op {
                BinaryOp::Add => {
                    let t = self.fresh_temp();
                    self.push(Instr::BinOp { dst: t, op: BinOp::Or, left: lv, right: rv, ty: IrType::Set(32) });
                    IrValue::Temp(t, IrType::Set(32))
                }
                BinaryOp::Multiply => {
                    let t = self.fresh_temp();
                    self.push(Instr::BinOp { dst: t, op: BinOp::And, left: lv, right: rv, ty: IrType::Set(32) });
                    IrValue::Temp(t, IrType::Set(32))
                }
                BinaryOp::Subtract => {
                    // A - B = A AND (NOT B) = A AND (B XOR all-ones)
                    let t_not = self.fresh_temp();
                    self.push(Instr::BinOp { dst: t_not, op: BinOp::Xor, left: rv, right: IrValue::ConstInt(-1, IrType::Set(32)), ty: IrType::Set(32) });
                    let t = self.fresh_temp();
                    self.push(Instr::BinOp { dst: t, op: BinOp::And, left: lv, right: IrValue::Temp(t_not, IrType::Set(32)), ty: IrType::Set(32) });
                    IrValue::Temp(t, IrType::Set(32))
                }
                BinaryOp::Divide => {
                    // A / B = A XOR B  (symmetric difference)
                    let t = self.fresh_temp();
                    self.push(Instr::BinOp { dst: t, op: BinOp::Xor, left: lv, right: rv, ty: IrType::Set(32) });
                    IrValue::Temp(t, IrType::Set(32))
                }
                // = and # on SET
                BinaryOp::Equal | BinaryOp::NotEqual => {
                    let ir_op = if op == BinaryOp::Equal { BinOp::Eq } else { BinOp::Ne };
                    let t = self.fresh_temp();
                    self.push(Instr::BinOp { dst: t, op: ir_op, left: lv, right: rv, ty: IrType::Bool });
                    IrValue::Temp(t, IrType::Bool)
                }
                _ => {
                    self.record_diagnostic(&format!("unsupported SET binary operator {op:?}"));
                    IrValue::ConstInt(0, IrType::Set(32))
                }
            };
        }

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

        // CP integer DIV and MOD require floor semantics (ENTIER(x/y)), not truncation-toward-zero.
        // Branchless expansion via sign-bit propagation (no branches / select needed):
        //   r = srem(x, y)
        //   rn_mask = ASH(r | -r, -bits)   → -1 if r != 0, else 0
        //   sd_mask = ASH(r XOR y, -bits)  → -1 if sign(r) != sign(y), else 0
        //   adj_mask = rn_mask AND sd_mask  → -1 iff adjustment required
        //   FloorMod = r + (adj_mask AND y)
        //   FloorDiv = sdiv(x, y) + adj_mask
        if matches!(op, BinaryOp::Div | BinaryOp::Mod)
            && matches!(ty, IrType::I64 | IrType::I32 | IrType::I16)
        {
            let bits: i128 = match &ty { IrType::I32 => 31, IrType::I16 => 15, _ => 63 };
            // t_r = srem(x, y)
            let t_r = self.fresh_temp();
            self.push(Instr::BinOp { dst: t_r, op: BinOp::Mod, left: lv.clone(), right: rv.clone(), ty: ty.clone() });
            // t_neg_r = 0 - r
            let t_neg_r = self.fresh_temp();
            self.push(Instr::BinOp { dst: t_neg_r, op: BinOp::Sub, left: IrValue::ConstInt(0, ty.clone()), right: IrValue::Temp(t_r, ty.clone()), ty: ty.clone() });
            // t_rn = r | -r  (high bit is set iff r != 0)
            let t_rn = self.fresh_temp();
            self.push(Instr::BinOp { dst: t_rn, op: BinOp::Or, left: IrValue::Temp(t_r, ty.clone()), right: IrValue::Temp(t_neg_r, ty.clone()), ty: ty.clone() });
            // t_rn_mask = ASH(t_rn, -bits)  → -1 if r != 0, else 0
            let t_rn_mask = self.fresh_temp();
            self.push(Instr::Ash { dst: t_rn_mask, value: IrValue::Temp(t_rn, ty.clone()), shift: IrValue::ConstInt(-bits, ty.clone()), ty: ty.clone() });
            // t_sd = r XOR y
            let t_sd = self.fresh_temp();
            self.push(Instr::BinOp { dst: t_sd, op: BinOp::Xor, left: IrValue::Temp(t_r, ty.clone()), right: rv.clone(), ty: ty.clone() });
            // t_sd_mask = ASH(t_sd, -bits)  → -1 if signs differ, else 0
            let t_sd_mask = self.fresh_temp();
            self.push(Instr::Ash { dst: t_sd_mask, value: IrValue::Temp(t_sd, ty.clone()), shift: IrValue::ConstInt(-bits, ty.clone()), ty: ty.clone() });
            // t_adj_mask = t_rn_mask AND t_sd_mask  → -1 iff adjustment needed
            let t_adj_mask = self.fresh_temp();
            self.push(Instr::BinOp { dst: t_adj_mask, op: BinOp::And, left: IrValue::Temp(t_rn_mask, ty.clone()), right: IrValue::Temp(t_sd_mask, ty.clone()), ty: ty.clone() });
            if matches!(op, BinaryOp::Mod) {
                // floor_r = r + (adj_mask AND y)
                let t_masked_y = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_masked_y, op: BinOp::And, left: IrValue::Temp(t_adj_mask, ty.clone()), right: rv, ty: ty.clone() });
                let t_result = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_result, op: BinOp::Add, left: IrValue::Temp(t_r, ty.clone()), right: IrValue::Temp(t_masked_y, ty.clone()), ty: ty.clone() });
                return IrValue::Temp(t_result, ty);
            } else {
                // floor_q = sdiv(x, y) + adj_mask
                let t_q = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_q, op: BinOp::Div, left: lv, right: rv, ty: ty.clone() });
                let t_result = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_result, op: BinOp::Add, left: IrValue::Temp(t_q, ty.clone()), right: IrValue::Temp(t_adj_mask, ty.clone()), ty: ty.clone() });
                return IrValue::Temp(t_result, ty);
            }
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
            BinaryOp::Is => {
                self.record_diagnostic("IS expression reached generic binary lowering path");
                debug_assert!(false, "lower_binary invariant violated: BinaryOp::Is should have returned via TypeCheck");
                return IrValue::ConstBool(false);
            }
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
            // ORD(c): CHAR/SHORTCHAR → INTEGER (zero-extend to i64)
            "ORD" => {
                let x = self.lower_expr(args.first()?);
                let dst = self.fresh_temp();
                self.push(Instr::Cast { dst, value: x, to_ty: IrType::I64 });
                Some(IrValue::Temp(dst, IrType::I64))
            }
            // CHR(n): INTEGER → CHAR (truncate to i32, unsigned char code point)
            "CHR" => {
                let x = self.lower_expr(args.first()?);
                let dst = self.fresh_temp();
                self.push(Instr::Cast { dst, value: x, to_ty: IrType::Char });
                Some(IrValue::Temp(dst, IrType::Char))
            }
            // SHORT: narrows one step in the type hierarchy
            //   CHAR → SHORTCHAR,  INTEGER → I32,  REAL → F32
            "SHORT" => {
                let x = self.lower_expr(args.first()?);
                let from_ty = x.ty();
                let to_ty = match &from_ty {
                    IrType::Char => IrType::ShortChar,
                    IrType::I64 => IrType::I32,
                    IrType::F64 => IrType::F32,
                    IrType::I32 => IrType::I16,
                    other => other.clone(),
                };
                if to_ty == from_ty {
                    return Some(x);
                }
                let dst = self.fresh_temp();
                self.push(Instr::Cast { dst, value: x, to_ty: to_ty.clone() });
                Some(IrValue::Temp(dst, to_ty))
            }
            // LONG: widens one step in the type hierarchy
            //   SHORTCHAR → CHAR,  I16/I32 → I64,  F32 → F64
            "LONG" => {
                let x = self.lower_expr(args.first()?);
                let from_ty = x.ty();
                let to_ty = match &from_ty {
                    IrType::ShortChar => IrType::Char,
                    IrType::I16 | IrType::I32 => IrType::I64,
                    IrType::F32 => IrType::F64,
                    other => other.clone(),
                };
                if to_ty == from_ty {
                    return Some(x);
                }
                let dst = self.fresh_temp();
                self.push(Instr::Cast { dst, value: x, to_ty: to_ty.clone() });
                Some(IrValue::Temp(dst, to_ty))
            }
            // ABS(x): absolute value
            //   Float: Cast with same float type (emitter uses llvm.fabs intrinsic)
            //   Integer: branchless (x XOR mask) - mask  where mask = x ASH -(bits-1)
            "ABS" => {
                let x = self.lower_expr(args.first()?);
                let ty = x.ty();
                match &ty {
                    IrType::F32 | IrType::F64 => {
                        // Use a dedicated "float abs" Cast (same from/to type → emitter uses fabs)
                        let dst = self.fresh_temp();
                        self.push(Instr::Cast { dst, value: x, to_ty: ty.clone() });
                        Some(IrValue::Temp(dst, ty))
                    }
                    _ => {
                        // Branchless integer abs: mask = x ASH -(bits-1), result = (x XOR mask) - mask
                        let bits: i128 = match &ty {
                            IrType::I32 => 31,
                            IrType::I16 => 15,
                            IrType::I8 => 7,
                            _ => 63,
                        };
                        let shift_const = IrValue::ConstInt(-bits, ty.clone());
                        let mask_t = self.fresh_temp();
                        self.push(Instr::Ash {
                            dst: mask_t,
                            value: x.clone(),
                            shift: shift_const,
                            ty: ty.clone(),
                        });
                        let xored_t = self.fresh_temp();
                        self.push(Instr::BinOp {
                            dst: xored_t,
                            op: BinOp::Xor,
                            left: x,
                            right: IrValue::Temp(mask_t, ty.clone()),
                            ty: ty.clone(),
                        });
                        let abs_t = self.fresh_temp();
                        self.push(Instr::BinOp {
                            dst: abs_t,
                            op: BinOp::Sub,
                            left: IrValue::Temp(xored_t, ty.clone()),
                            right: IrValue::Temp(mask_t, ty.clone()),
                            ty: ty.clone(),
                        });
                        Some(IrValue::Temp(abs_t, ty))
                    }
                }
            }
            // BITS(x: INTEGER): SET  — §10.3
            //   Interprets the low 32 bits of x as a bitset.
            //   Implementation: truncate I64 → Set(32).
            "BITS" => {
                let x = self.lower_expr(args.first()?);
                let dst = self.fresh_temp();
                self.push(Instr::Cast { dst, value: x, to_ty: IrType::Set(32) });
                Some(IrValue::Temp(dst, IrType::Set(32)))
            }
            // ENTIER(x): floor of x converted to LONGINT (i64).
            //   Maps to llvm.floor intrinsic + fptosi.
            "ENTIER" => {
                let x = self.lower_expr(args.first()?);
                let dst = self.fresh_temp();
                self.push(Instr::Entier { dst, value: x });
                Some(IrValue::Temp(dst, IrType::I64))
            }
            // CAP(x): Latin-1 letter → corresponding capital; other chars unchanged.
            //   Branchless: is_lower = (x >= 'a') AND (x <= 'z')
            //   result = x - (is_lower ? 32 : 0)
            //   Works for both CHAR (i32/u32) and SHORTCHAR (i8).
            "CAP" => {
                let x = self.lower_expr(args.first()?);
                let ty = x.ty();
                let a_ord: i128 = 0x61; // 'a'
                let z_ord: i128 = 0x7A; // 'z'
                // ge_a = (x >= 'a')
                let t_ge = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_ge, op: BinOp::Ge, left: x.clone(), right: IrValue::ConstInt(a_ord, ty.clone()), ty: IrType::Bool });
                // le_z = (x <= 'z')
                let t_le = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_le, op: BinOp::Le, left: x.clone(), right: IrValue::ConstInt(z_ord, ty.clone()), ty: IrType::Bool });
                // is_lower = ge_a AND le_z  (bool AND bool)
                let t_is = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_is, op: BinOp::And, left: IrValue::Temp(t_ge, IrType::Bool), right: IrValue::Temp(t_le, IrType::Bool), ty: IrType::Bool });
                // extend bool to the char type: 1 → 1, 0 → 0
                let t_flag = self.fresh_temp();
                self.push(Instr::Cast { dst: t_flag, value: IrValue::Temp(t_is, IrType::Bool), to_ty: ty.clone() });
                // delta = flag * 32
                let t_delta = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_delta, op: BinOp::Mul, left: IrValue::Temp(t_flag, ty.clone()), right: IrValue::ConstInt(32, ty.clone()), ty: ty.clone() });
                // result = x - delta
                let t_result = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_result, op: BinOp::Sub, left: x, right: IrValue::Temp(t_delta, ty.clone()), ty: ty.clone() });
                Some(IrValue::Temp(t_result, ty))
            }
            // LEN(v): length of the array in its first dimension.
            //   - Fixed-size local/global array (IrType::Array { len, .. }): static `len`.
            //   - Open-array parameter: load the hidden `<name>$len: I64` companion
            //     param injected by the open-array fat-pointer ABI (see lower_procedure
            //     and lower_call_args). Multi-dim LEN(v, dim) is not yet supported.
            "LEN" => {
                let arg = args.first()?;
                let des = match arg {
                    Expr::Designator(d) => d,
                    _ => {
                        self.record_diagnostic("LEN: argument must be an array designator");
                        return Some(IrValue::ConstInt(0, IrType::I64));
                    }
                };
                // Open-array param: bare local name with a `<name>$len` companion.
                if des.selectors.is_empty() && des.base.module.is_none() {
                    let hidden = format!("{}{OPEN_ARRAY_LEN_SUFFIX}", des.base.name);
                    if self.proc.params.iter().any(|(n, _)| n == &hidden) {
                        let addr = IrValue::GlobalRef(hidden, IrType::Ref(Box::new(IrType::I64)));
                        let t = self.fresh_temp();
                        self.push(Instr::Load { dst: t, addr, ty: IrType::I64 });
                        return Some(IrValue::Temp(t, IrType::I64));
                    }
                }
                // Dynamic open-array deref: `LEN(p^)` where p is a
                // POINTER TO ARRAY OF T. Read the length back from the
                // 8-byte header that NewArray stored at `p - 8`. We
                // strip the trailing Dereference selector and load the
                // pointer value, then emit `LoadRaw (p_int - 8)`.
                if let Some(Selector::Dereference) = des.selectors.last() {
                    let prefix_des = Designator {
                        span: des.span,
                        base: des.base.clone(),
                        selectors: des.selectors[..des.selectors.len() - 1].to_vec(),
                    };
                    let prefix_ty = self.designator_ir_type(&prefix_des);
                    let is_dyn_array_ptr = match &prefix_ty {
                        Some(IrType::Ptr(_)) | Some(IrType::UntaggedPtr(_)) => true,
                        Some(IrType::Named(n)) => matches!(
                            self.resolve_named_as_ptr_ir_type(n),
                            Some(IrType::Ptr(_)) | Some(IrType::UntaggedPtr(_))
                        ),
                        _ => false,
                    };
                    if is_dyn_array_ptr {
                        let p_val = self.lower_expr(&Expr::Designator(prefix_des));
                        // Cast the pointer to i64 so we can subtract.
                        let p_int = self.fresh_temp();
                        self.push(Instr::BitCast {
                            dst: p_int,
                            value: p_val,
                            ty: IrType::I64,
                        });
                        let header_addr = self.fresh_temp();
                        self.push(Instr::BinOp {
                            dst: header_addr,
                            op: BinOp::Sub,
                            left: IrValue::Temp(p_int, IrType::I64),
                            right: IrValue::ConstInt(8, IrType::I64),
                            ty: IrType::I64,
                        });
                        let t = self.fresh_temp();
                        self.push(Instr::LoadRaw {
                            dst: t,
                            addr: IrValue::Temp(header_addr, IrType::I64),
                            ty: IrType::I64,
                        });
                        return Some(IrValue::Temp(t, IrType::I64));
                    }
                }
                let ir_ty = self.designator_ir_type(des)?;
                if let IrType::Array { len, .. } = ir_ty {
                    return Some(IrValue::ConstInt(len as i128, IrType::I64));
                }
                self.record_diagnostic("LEN: argument is not a fixed-size array or open-array param");
                Some(IrValue::ConstInt(0, IrType::I64))
            }
            // MAX(T) / MIN(T): single-arg form takes a *type* identifier and
            // returns the type's largest / smallest value as a constant. CP spec
            // §10.3. Used heavily by overflow-aware integer parsers.
            //
            //   MIN(REAL)         smallest positive normal        (IEEE-754 MIN_POSITIVE)
            //   MAX(REAL)         largest finite value            (IEEE-754 MAX)
            //   MIN(INTEGER)..    standard signed range
            //   MIN(SET) = 0      MAX(SET) = 31  (returned as INTEGER)
            //   MIN(CHAR) = 0X    MAX(CHAR) = 10FFFFX  (Unicode scalar range)
            "MAX" if args.len() == 1 => {
                return self.min_max_one_arg_resolving(args.first()?, /*max=*/ true);
            }
            "MIN" if args.len() == 1 => {
                return self.min_max_one_arg_resolving(args.first()?, /*max=*/ false);
            }
            // MAX(x, y): two-argument maximum (branchless via sign-bit mask)            //   diff = x - y
            //   mask = ASH(diff, -(bits-1))  → -1 if x < y, 0 if x >= y
            //   not_mask = mask XOR -1
            //   result = y + (diff AND not_mask)
            "MAX" if args.len() >= 2 => {
                let x = self.lower_expr(args.first()?);
                let y = self.lower_expr(args.get(1)?);
                let ty = x.ty();
                let bits: i128 = match &ty { IrType::I32 => 31, IrType::I16 => 15, IrType::I8 => 7, _ => 63 };
                let t_diff = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_diff, op: BinOp::Sub, left: x, right: y.clone(), ty: ty.clone() });
                let t_mask = self.fresh_temp();
                self.push(Instr::Ash { dst: t_mask, value: IrValue::Temp(t_diff, ty.clone()), shift: IrValue::ConstInt(-bits, ty.clone()), ty: ty.clone() });
                let t_not_mask = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_not_mask, op: BinOp::Xor, left: IrValue::Temp(t_mask, ty.clone()), right: IrValue::ConstInt(-1, ty.clone()), ty: ty.clone() });
                let t_masked = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_masked, op: BinOp::And, left: IrValue::Temp(t_diff, ty.clone()), right: IrValue::Temp(t_not_mask, ty.clone()), ty: ty.clone() });
                let t_result = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_result, op: BinOp::Add, left: y, right: IrValue::Temp(t_masked, ty.clone()), ty: ty.clone() });
                Some(IrValue::Temp(t_result, ty))
            }
            // MIN(x, y): two-argument minimum (branchless via sign-bit mask)
            //   diff = x - y
            //   mask = ASH(diff, -(bits-1))  → -1 if x < y, 0 if x >= y
            //   not_mask = mask XOR -1
            //   result = x - (diff AND not_mask)
            "MIN" if args.len() >= 2 => {
                let x = self.lower_expr(args.first()?);
                let y = self.lower_expr(args.get(1)?);
                let ty = x.ty();
                let bits: i128 = match &ty { IrType::I32 => 31, IrType::I16 => 15, IrType::I8 => 7, _ => 63 };
                let t_diff = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_diff, op: BinOp::Sub, left: x.clone(), right: y, ty: ty.clone() });
                let t_mask = self.fresh_temp();
                self.push(Instr::Ash { dst: t_mask, value: IrValue::Temp(t_diff, ty.clone()), shift: IrValue::ConstInt(-bits, ty.clone()), ty: ty.clone() });
                let t_not_mask = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_not_mask, op: BinOp::Xor, left: IrValue::Temp(t_mask, ty.clone()), right: IrValue::ConstInt(-1, ty.clone()), ty: ty.clone() });
                let t_masked = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_masked, op: BinOp::And, left: IrValue::Temp(t_diff, ty.clone()), right: IrValue::Temp(t_not_mask, ty.clone()), ty: ty.clone() });
                let t_result = self.fresh_temp();
                self.push(Instr::BinOp { dst: t_result, op: BinOp::Sub, left: x, right: IrValue::Temp(t_masked, ty.clone()), ty: ty.clone() });
                Some(IrValue::Temp(t_result, ty))
            }
            _ => None,
        }
    }

    /// MAX(T)/MIN(T) for a type-name argument. Resolves user-defined
    /// transparent aliases (e.g. `TYPE DoubleDigit = INTEGER`) to the
    /// underlying builtin name before delegating to [`min_max_one_arg`].
    /// Without this, `MAX(DoubleDigit)` would reach LLVM as an unknown
    /// direct callee.
    fn min_max_one_arg_resolving(&self, arg: &Expr, max: bool) -> Option<IrValue> {
        if let Some(direct) = min_max_one_arg(arg, max) {
            return Some(direct);
        }
        let name = match arg {
            Expr::Designator(des)
                if des.base.module.is_none() && des.selectors.is_empty() =>
            {
                des.base.name.as_str()
            }
            _ => return None,
        };
        let mut current = name.to_string();
        for _ in 0..16 {
            let symbol = self
                .module_symbols
                .iter()
                .find(|s| s.name == current && s.kind == SymbolKind::Type)?;
            match symbol.declared_type.as_ref()? {
                SemanticType::Builtin(builtin) => {
                    let synth_name = match builtin {
                        BuiltinType::Byte => "BYTE",
                        BuiltinType::ShortInt => "SHORTINT",
                        BuiltinType::IntShort => "INTSHORT",
                        BuiltinType::Integer => "INTEGER",
                        BuiltinType::LongInt => "LONGINT",
                        BuiltinType::Real => "REAL",
                        BuiltinType::ShortReal => "SHORTREAL",
                        BuiltinType::Char => "CHAR",
                        BuiltinType::ShortChar => "SHORTCHAR",
                        BuiltinType::Set => "SET",
                        _ => return None,
                    };
                    let synth = Expr::Designator(Designator {
                        span: match arg {
                            Expr::Designator(d) => d.span,
                            _ => return None,
                        },
                        base: QualIdent {
                            module: None,
                            name: synth_name.to_string(),
                            span: match arg {
                                Expr::Designator(d) => d.base.span,
                                _ => return None,
                            },
                        },
                        selectors: Vec::new(),
                    });
                    return min_max_one_arg(&synth, max);
                }
                SemanticType::Named { module: None, name: alias_name, .. } => {
                    current = alias_name.clone();
                }
                _ => return None,
            }
        }
        None
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
                // Named pointer alias — resolve through `Bag = POINTER TO
                // ARRAY OF T` to find the element type. Same pattern as the
                // designator_addr path uses for the same case.
                (Selector::Index(_), IrType::Named(n)) => {
                    match self.resolve_named_as_ptr_ir_type(&n) {
                        Some(IrType::Ptr(elem)) | Some(IrType::UntaggedPtr(elem)) => *elem,
                        _ => IrType::Opaque("indexed-named".to_string()),
                    }
                }
                (Selector::Dereference, IrType::Named(n)) => {
                    match self.resolve_named_as_ptr_ir_type(&n) {
                        Some(IrType::Ptr(inner)) | Some(IrType::UntaggedPtr(inner)) => *inner,
                        _ => IrType::Opaque("deref-named".to_string()),
                    }
                }
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
                // NEW(ptr_var, len) — allocate a heap buffer of `len`
                // elements for a `POINTER TO ARRAY OF T` target.
                let Some(Expr::Designator(target)) = args.first() else {
                    return false;
                };
                // Resolve the pointer alias to get the record/element type to allocate.
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

                // Two-arg form: NEW(ptr, len) for POINTER TO ARRAY OF T.
                // The "record_ty" we resolved above is actually the element
                // type for this case (the open-array's element).
                if args.len() == 2 {
                    let len_val = self.lower_expr(&args[1]);
                    let dst = self.fresh_temp();
                    self.push(Instr::NewArray {
                        dst,
                        elem_ty: record_ty.clone(),
                        len: len_val,
                    });
                    let ptr_ir_ty = match &ptr_sym_ty {
                        IrType::Named(n) => self
                            .resolve_named_as_ptr_ir_type(n)
                            .unwrap_or_else(|| IrType::Ptr(Box::new(record_ty.clone()))),
                        other => other.clone(),
                    };
                    let target_addr = self.designator_addr(target);
                    self.push(Instr::Store {
                        addr: target_addr,
                        value: IrValue::Temp(dst, ptr_ir_ty),
                    });
                    return true;
                }

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
            // INCL(v, x): v := v + {x}  →  v := v OR (1 << x)
            "INCL" => {
                let Some(Expr::Designator(target)) = args.first() else { return false; };
                let Some(bit_expr) = args.get(1) else { return false; };
                let addr = self.designator_addr(target);
                let current = self.fresh_temp();
                self.push(Instr::Load { dst: current, addr: addr.clone(), ty: IrType::Set(32) });
                let bit = self.lower_expr(bit_expr);
                let one = IrValue::ConstInt(1, IrType::Set(32));
                let bit64 = self.fresh_temp();
                self.push(Instr::Cast { dst: bit64, value: bit, to_ty: IrType::Set(32) });
                let shifted = self.fresh_temp();
                self.push(Instr::BinOp { dst: shifted, op: BinOp::Shl, left: one, right: IrValue::Temp(bit64, IrType::Set(32)), ty: IrType::Set(32) });
                let result = self.fresh_temp();
                self.push(Instr::BinOp { dst: result, op: BinOp::Or, left: IrValue::Temp(current, IrType::Set(32)), right: IrValue::Temp(shifted, IrType::Set(32)), ty: IrType::Set(32) });
                self.push(Instr::Store { addr, value: IrValue::Temp(result, IrType::Set(32)) });
                true
            }
            // EXCL(v, x): v := v - {x}  →  v := v AND NOT (1 << x)  →  v AND ((1<<x) XOR -1)
            "EXCL" => {
                let Some(Expr::Designator(target)) = args.first() else { return false; };
                let Some(bit_expr) = args.get(1) else { return false; };
                let addr = self.designator_addr(target);
                let current = self.fresh_temp();
                self.push(Instr::Load { dst: current, addr: addr.clone(), ty: IrType::Set(32) });
                let bit = self.lower_expr(bit_expr);
                let one = IrValue::ConstInt(1, IrType::Set(32));
                let bit64 = self.fresh_temp();
                self.push(Instr::Cast { dst: bit64, value: bit, to_ty: IrType::Set(32) });
                let shifted = self.fresh_temp();
                self.push(Instr::BinOp { dst: shifted, op: BinOp::Shl, left: one, right: IrValue::Temp(bit64, IrType::Set(32)), ty: IrType::Set(32) });
                let mask = self.fresh_temp();
                self.push(Instr::BinOp { dst: mask, op: BinOp::Xor, left: IrValue::Temp(shifted, IrType::Set(32)), right: IrValue::ConstInt(-1, IrType::Set(32)), ty: IrType::Set(32) });
                let result = self.fresh_temp();
                self.push(Instr::BinOp { dst: result, op: BinOp::And, left: IrValue::Temp(current, IrType::Set(32)), right: IrValue::Temp(mask, IrType::Set(32)), ty: IrType::Set(32) });
                self.push(Instr::Store { addr, value: IrValue::Temp(result, IrType::Set(32)) });
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
                // Coerce rhs to match the slot's element type when they differ.
                // Handles e.g. SHORT(INTEGER)→I32 stored into a SHORTINT:I16 slot.
                //
                // For VAR/OUT/IN params the addr is `Ref(Ref(T))` (the outer Ref is
                // the slot for the param itself; the inner Ref is the runtime
                // pointer to the caller's variable). The actual storage element is
                // T, not Ref(T), so peel through nested Refs to find it.
                let mut slot_ty = match addr.ty() {
                    IrType::Ref(inner) | IrType::Ptr(inner) => Some(*inner),
                    _ => None,
                };
                while let Some(IrType::Ref(inner)) = slot_ty.clone() {
                    slot_ty = Some(*inner);
                }

                // String-literal initialization of a fixed-size CHAR/SHORTCHAR
                // array (`digits := "0123456789ABCDEF"`). The default Cast path
                // can't lower ptr -> [N x char]; instead emit a memcpy from the
                // string literal's private global into the array slot.
                if let (Some(IrType::Array { element, len }), IrValue::ConstStr(s, lit_elem)) =
                    (slot_ty.clone(), &rhs)
                {
                    let elem = *element.clone();
                    let elem_size: usize = match elem {
                        IrType::Char => 4,
                        IrType::ShortChar | IrType::U8 | IrType::I8 => 1,
                        _ => 0,
                    };
                    if elem_size > 0 {
                        // Retype the literal's element to match the destination
                        // (e.g. literal defaults to CHAR but target is SHORTCHAR).
                        let src = if *lit_elem != elem {
                            IrValue::ConstStr(s.clone(), elem.clone())
                        } else {
                            rhs.clone()
                        };
                        let lit_units = s.chars().count() + 1;	// + NUL terminator
                        let copy_units = lit_units.min(len as usize);
                        let bytes = (copy_units * elem_size) as i128;

                        // Convert pointer-typed dst/src to i64 addresses for MemCopy.
                        let dst_t = self.fresh_temp();
                        self.push(Instr::AddrOf { dst: dst_t, sym: addr.clone() });
                        let src_t = self.fresh_temp();
                        self.push(Instr::AddrOf { dst: src_t, sym: src });
                        self.push(Instr::MemCopy {
                            dst: IrValue::Temp(dst_t, IrType::I64),
                            src: IrValue::Temp(src_t, IrType::I64),
                            len: IrValue::ConstInt(bytes, IrType::I64),
                        });
                        return;
                    }
                }
                let rhs = if let Some(slot_ty) = slot_ty {
                    if slot_ty != rhs.ty() {
                        let t = self.fresh_temp();
                        self.push(Instr::Cast { dst: t, value: rhs, to_ty: slot_ty.clone() });
                        IrValue::Temp(t, slot_ty)
                    } else {
                        rhs
                    }
                } else {
                    rhs
                };
                self.push(Instr::Store { addr, value: rhs });
            }

            Statement::ProcedureCall { designator, .. } => {
                if !self.lower_inc_dec_statement(designator)
                    && !self.lower_system_statement(designator)
                    && !self.lower_builtin_statement(designator)
                {
                    // In CP, a parameterless procedure may be called without `()`.
                    // Detect: qualified or local name with no selectors that resolves
                    // to a procedure type → emit a zero-arg call directly.
                    let is_bare_proc_call = {
                        let (mod_opt, name) = match &designator.base {
                            QualIdent { module: Some(m), name, .. } => (Some(m.as_str()), name.as_str()),
                            QualIdent { name, .. } => (None, name.as_str()),
                        };
                        designator.selectors.is_empty() && (
                            mod_opt.is_some() ||
                            self.symbols.iter().rev()
                                .find(|s| s.name == name)
                                .map(|s| matches!(s.kind, SymbolKind::Procedure))
                                .unwrap_or(false)
                        )
                    };
                    if is_bare_proc_call {
                        let (mod_opt, name) = match &designator.base {
                            QualIdent { module: Some(m), name, .. } => (Some(m.clone()), name.clone()),
                            QualIdent { name, .. } => (None, name.clone()),
                        };
                        let final_ty = self.designator_ir_type(designator)
                            .unwrap_or_else(|| IrType::Opaque("unresolved".to_string()));
                        let callee = match mod_opt {
                            Some(m) => IrValue::ImportRef(m, name, final_ty),
                            None => IrValue::GlobalRef(name, final_ty),
                        };
                        let ret_ty = self.callee_return_type(&callee);
                        self.push(Instr::Call { dst: None, callee, args: vec![], ret_ty });
                    } else {
                        let _ = self.lower_designator(designator);
                    }
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

        // Pre-evaluate step (always a constant in CP) to determine loop direction.
        // A negative step means descending: continue while var >= end (Ge).
        // A positive/absent step means ascending: continue while var <= end (Le).
        let is_descending = step.map_or(false, |e| matches!(e, Expr::Unary { op: UnaryOp::Minus, .. }));
        let cond_op = if is_descending { BinOp::Ge } else { BinOp::Le };

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
            op: cond_op,
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
                let mut val = self.lower_expr(ret_expr);
                // Numeric/char widening: coerce to the procedure's declared
                // return type when the expression's type is narrower or
                // smaller. Without this, `RETURN MIN(SHORTINT)` from an
                // `: INTEGER` proc performs an i16 store into the i64 result
                // slot, leaving the upper bytes undefined.
                //
                // We restrict the cast to scalar numeric / char types so we
                // don't try to coerce across pointers, records, arrays, etc.
                let ret_ty = self.proc.ret_ty.clone();
                if val.ty() != ret_ty
                    && is_scalar_castable(&val.ty())
                    && is_scalar_castable(&ret_ty)
                {
                    let t = self.fresh_temp();
                    self.push(Instr::Cast { dst: t, value: val, to_ty: ret_ty.clone() });
                    val = IrValue::Temp(t, ret_ty);
                }
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
    //
    // Open-array upvalues also get a companion `<name>$len: Ref(I64)` upvalue so that
    // `LEN(captured_open_array)` inside the nested proc can resolve.
    let mut upvalue_params: Vec<(String, IrType)> = Vec::new();
    for (name, ty) in &sema_proc.upvalues {
        upvalue_params.push((name.clone(), IrType::Ref(Box::new(map_semantic_type(ty)))));
        if is_open_array(ty) {
            upvalue_params.push((
                format!("{name}{OPEN_ARRAY_LEN_SUFFIX}"),
                IrType::Ref(Box::new(IrType::I64)),
            ));
        }
    }

    let mut params: Vec<(String, IrType)> = upvalue_params;
    params.extend(receiver_param.iter().cloned());
    for param in &sema_proc.signature.parameters {
        let base_ty = map_semantic_type(&param.ty);
        let ir_ty = match param.mode {
            Some(ParamMode::Var) | Some(ParamMode::Out) => {
                IrType::Ref(Box::new(base_ty))
            }
            Some(ParamMode::In) => IrType::Ref(Box::new(base_ty)),
            None => base_ty,
        };
        // For each declared name in this param group, push the user param.
        // Open-array params get a hidden `<name>$len: I64` immediately after,
        // implementing CP's decomposed fat-pointer ABI.
        let is_open = is_open_array(&param.ty);
        for name in &param.names {
            params.push((name.clone(), ir_ty.clone()));
            if is_open {
                params.push((format!("{name}{OPEN_ARRAY_LEN_SUFFIX}"), IrType::I64));
            }
        }
    }

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
        if let Some(result_addr) = result_slot {
            let t = ctx.fresh_temp();
            ctx.push(Instr::Load {
                dst: t,
                addr: result_addr,
                ty: ret_ty.clone(),
            });
            Terminator::Ret { value: IrValue::Temp(t, ret_ty) }
        } else {
            ctx.record_diagnostic("non-void procedure missing result slot at function exit");
            debug_assert!(false, "lower_procedure invariant violated: non-void procedure missing result slot");
            Terminator::Trap { kind: TrapKind::Assert }
        }
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
        let filename = format!("{module}.cp");
        let mut candidate_paths: Vec<PathBuf> = Vec::new();

        // 1. Sibling of the importing source file.
        let import_root = IMPORT_SEARCH_ROOT.with(|root| root.borrow().clone());
        if let Some(base) = &import_root {
            candidate_paths.push(base.join(&filename));
            // 2. Walk up from the importing file's dir until we find a "Mod"
            //    directory; search it recursively. Mirrors the loader's
            //    `resolve_import_source` so a module under `Mod/Tests/` can
            //    import a module that lives in `Mod/` directly.
            let mut dir: &Path = base.as_path();
            loop {
                if dir.file_name().and_then(|n| n.to_str()) == Some("Mod") {
                    if let Some(hit) = find_cp_in_dir_recursive(dir, &filename) {
                        candidate_paths.push(hit);
                    }
                    break;
                }
                match dir.parent() {
                    Some(p) => dir = p,
                    None => break,
                }
            }
        }
        // 3. Cwd-relative `Mod/` fallback (legacy).
        candidate_paths.push(Path::new("Mod").join(&filename));

        let mut imported_module = None;
        for path in candidate_paths {
            let Ok(ast) = read_module_ast(&path) else {
                continue;
            };
            imported_module = Some(analyze_module_ast(&ast));
            break;
        }

        cache.insert(module.to_string(), imported_module?);
    }
    cache.get(module)
}

/// Decompose a procedure signature into per-position `(mode, type)`
/// vectors with one entry per scalar parameter (parameters declared as
/// `a, b, c: T` expand to three entries).
///
/// Used by both direct-call and method-call argument lowering so the
/// open-array fat-pointer ABI, VAR-mode address passing, and
/// SHORTCHAR widening are applied uniformly.
fn flatten_param_modes_and_types(
    proc_ty: Option<&newcp_sema::ProcedureType>,
) -> (Vec<Option<ParamMode>>, Vec<SemanticType>) {
    let modes = proc_ty
        .map(|pt| {
            pt.parameters
                .iter()
                .flat_map(|p| std::iter::repeat_n(p.mode, p.names.len()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let types = proc_ty
        .map(|pt| {
            pt.parameters
                .iter()
                .flat_map(|p| std::iter::repeat_n(p.ty.clone(), p.names.len()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    (modes, types)
}

/// Walk the inheritance chain of `record_name` (within `symbols`) and
/// return `(declaring_module, declaring_type_name, method_name)` triples
/// in vtable-slot order. Used to seed the vtable of a cross-module-
/// extended concrete record so that override slot indices line up with
/// the imported base.
///
/// `declaring_module` is `Some(module_name)` when the slot's body lives
/// in another JIT module (cross-module inheritance), `None` when the
/// slot's body is in the current module being compiled.
///
/// Each triple represents the function-pointer text the seed vtable
/// holds for that slot — typically the abstract-method shim emitted by
/// the declaring module. Concrete subclasses overwrite these slots.
fn collect_inherited_method_names(
    record_name: &str,
    symbols: &[SemanticSymbol],
    declaring_module: Option<&str>,
) -> Vec<(Option<String>, String, String)> {
    use newcp_sema::SymbolKind;
    let Some(sym) = symbols.iter().find(|s| s.kind == SymbolKind::Type && s.name == record_name)
    else { return Vec::new() };
    let Some(SemanticType::Record { base, methods, .. }) = sym.declared_type.as_ref()
    else { return Vec::new() };

    // Inherited slots come first. Recursion preserves the current
    // `declaring_module` for local hops; a cross-module hop replaces it
    // with the imported module's name.
    let mut result: Vec<(Option<String>, String, String)> = match base.as_deref() {
        Some(SemanticType::Named { name, module: None, .. }) => {
            collect_inherited_method_names(name, symbols, declaring_module)
        }
        Some(SemanticType::Named { name, module: Some(m), .. }) => {
            find_imported_module_sema(m)
                .map(|sema| collect_inherited_method_names(name, &sema.symbols, Some(m.as_str())))
                .unwrap_or_default()
        }
        _ => Vec::new(),
    };

    // NEW methods declared on this record extend the vtable; overrides
    // simply replace existing slots and don't add new ones.
    for method in methods {
        if method.signature.is_new {
            result.push((
                declaring_module.map(str::to_string),
                record_name.to_string(),
                method.name.clone(),
            ));
        }
    }

    result
}

/// Recursively qualify any internal `Named { module: None, name: T }`
/// references inside `ty` to `Named { module: Some(<module_name>), .. }`
/// when `T` is one of `local_type_names` (the type names declared at
/// the top level of the source module). Mirrors sema's
/// `qualify_local_named_refs` and is required so the IR layer can route
/// downstream symbol lookups to the right imported module.
fn qualify_local_named_refs_in_sem_type(
    ty: &mut SemanticType,
    module_name: &str,
    local_type_names: &std::collections::HashSet<String>,
) {
    match ty {
        SemanticType::Named { module, name, .. } => {
            if module.is_none() && local_type_names.contains(name) {
                *module = Some(module_name.to_string());
            }
        }
        SemanticType::Array { element_type, .. } => {
            qualify_local_named_refs_in_sem_type(element_type, module_name, local_type_names);
        }
        SemanticType::Record { base, fields, .. } => {
            if let Some(base) = base.as_deref_mut() {
                qualify_local_named_refs_in_sem_type(base, module_name, local_type_names);
            }
            for field in fields {
                qualify_local_named_refs_in_sem_type(&mut field.ty, module_name, local_type_names);
            }
        }
        SemanticType::Pointer { target, .. } => {
            qualify_local_named_refs_in_sem_type(target, module_name, local_type_names);
        }
        _ => {}
    }
}

/// Locate and analyze an imported module's source by name, using the
/// same search strategy as `load_cached_import` (sibling-of-importing-
/// source via `IMPORT_SEARCH_ROOT`, walk up to a `Mod/` parent, then
/// the cwd-relative `Mod/` fallback).
///
/// Used by `flatten_sem_type_fields` for cross-module record bases —
/// the function is static (no `&mut self`) so it can't reach the
/// per-LowerCtx import cache, which is fine here because field-flatten
/// happens once per record traversal during planning.
fn find_imported_module_sema(module_name: &str) -> Option<SemanticModule> {
    let filename = format!("{module_name}.cp");
    let mut candidate_paths: Vec<PathBuf> = Vec::new();

    let import_root = IMPORT_SEARCH_ROOT.with(|root| root.borrow().clone());
    if let Some(base) = &import_root {
        candidate_paths.push(base.join(&filename));
        let mut dir: &Path = base.as_path();
        loop {
            if dir.file_name().and_then(|n| n.to_str()) == Some("Mod") {
                if let Some(hit) = find_cp_in_dir_recursive(dir, &filename) {
                    candidate_paths.push(hit);
                }
                break;
            }
            match dir.parent() {
                Some(p) => dir = p,
                None => break,
            }
        }
    }
    candidate_paths.push(Path::new("Mod").join(&filename));

    for path in candidate_paths {
        if let Ok(ast) = read_module_ast(&path) {
            return Some(analyze_module_ast(&ast));
        }
    }
    None
}

/// Recursively search `dir` for a file named `filename`. Used by
/// `load_cached_import` to resolve sibling-Mod imports across directory
/// nesting (e.g. `Mod/Tests/Foo.cp` importing `Mod/Bar.cp`).
fn find_cp_in_dir_recursive(dir: &Path, filename: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            return Some(path);
        }
    }
    for sub in subdirs {
        if let Some(hit) = find_cp_in_dir_recursive(&sub, filename) {
            return Some(hit);
        }
    }
    None
}

/// Parse a Component Pascal integer literal.
///
/// - Decimal: plain digits, e.g. `255`, `1000`
/// - Hex with H suffix: `digit {hexDigit} H` — interpreted as 32-bit signed (sign-extends to i128)
/// - Hex with L suffix: `digit {hexDigit} L` — interpreted as 64-bit signed
///
/// Spec §3: `0DH → 13`, `0FFFF0000H → -65536`, `0FFFF0000L → 4294901760`.
fn parse_cp_integer_literal(s: &str) -> i128 {
    if let Some(hex) = s.strip_suffix('H') {
        // 32-bit interpretation: sign-extend i32 → i128
        let raw = u32::from_str_radix(hex, 16).unwrap_or(0);
        (raw as i32) as i128
    } else if let Some(hex) = s.strip_suffix('L') {
        // 64-bit interpretation: sign-extend i64 → i128
        let raw = u64::from_str_radix(hex, 16).unwrap_or(0);
        (raw as i64) as i128
    } else {
        s.parse::<i128>().unwrap_or(0)
    }
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

    // Synthesize the module-body procedure from `MODULE Foo; ... BEGIN <body> END Foo.`.
    // CP runs this once at module load (after all imports' bodies have run);
    // it's where module-level VAR initialisation lives. Without it, VARs stay
    // at their zero default — which silently breaks any module that depends
    // on them (e.g. Strings.cp's `digits`, `maxExp`, `factor`).
    //
    // We emit the body as an exported procedure named "body". The full LLVM
    // symbol becomes `<ModName>.body` — matches the existing `init_routine`
    // convention. The loader looks this symbol up after materialization and
    // JIT-calls it in dependency order.
    if let Some(body_stmts) = ast.body.as_ref() {
        if !body_stmts.is_empty() {
            let body_proc = lower_module_body(
                &sema.name,
                body_stmts,
                system_qualifiers.clone(),
                &sema.symbols,
                &sema.procedures,
            );
            procedures.push(body_proc);
        }
    }

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

/// Synthesize the module-body procedure. Mirrors `lower_procedure` for a
/// param-less, void-return proc whose body is the module's BEGIN..END.
fn lower_module_body(
    module_name: &str,
    body: &[Statement],
    system_qualifiers: Vec<String>,
    module_symbols: &[SemanticSymbol],
    all_sema_procs: &[SemanticProcedure],
) -> IrProcedure {
    let mut proc = IrProcedure::new(
        "body".to_string(),
        /*exported=*/ true,
        Vec::new(),
        IrType::Void,
    );
    let entry = proc.alloc_block();
    let function_exit = proc.alloc_block();
    proc.entry = entry;
    proc.exit = function_exit;

    // Suppress unused-arg warnings — kept as parameters in case future
    // module-body lowering needs the import-cache or top-level-proc list.
    let _ = (module_name, all_sema_procs);

    let mut ctx = LowerCtx::new(
        proc,
        entry,
        function_exit,
        /*result_slot=*/ None,
        module_symbols.to_vec(),
        system_qualifiers,
        module_symbols,
    );
    // outer_proc_name stays None: the body isn't a nested proc.
    // nested_proc_upvalues / return_types stay empty: the body calls
    // top-level procs via their flat (unmangled) names through the normal
    // is_direct_callee path.

    ctx.switch_to(entry);
    ctx.lower_statements(body);
    ctx.set_term(Terminator::Br { target: function_exit });

    ctx.switch_to(function_exit);
    ctx.set_term(Terminator::RetVoid);

    let mut proc = ctx.proc;
    proc.prune_unreachable();
    proc.compute_rpo();
    proc
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

        // Record direct base name (local only for now — cross-module
        // bases stay None here; vtable construction below handles them
        // separately by reaching into the imported module's sema).
        let base_name: Option<String> = base.as_deref().and_then(|b| match b {
            SemanticType::Named { name, module: None, .. } => Some(name.clone()),
            _ => None,
        });
        bases.insert(sym.name.clone(), base_name.clone());

        // The seed vtable for this type: if the base is local we already
        // built its vtable in this loop iteration order (sema collects
        // bases before subclasses for the modules we ship); if the base
        // is imported, we synthesise a placeholder vec sized to the
        // imported base's vtable length so override-slot indices stay
        // correct. The placeholder entries are abstract-method shims —
        // they should be overwritten by every concrete subclass.
        let cross_module_base: Option<(String, String)> = base.as_deref().and_then(|b| match b {
            SemanticType::Named { name, module: Some(m), .. } => {
                Some((m.clone(), name.clone()))
            }
            _ => None,
        });

        let mut vtable: Vec<String> = if let Some(bn) = base_name.as_deref() {
            vtables.get(bn).cloned().unwrap_or_default()
        } else if let Some((module_name, base_record_name)) = cross_module_base.as_ref() {
            // Pull the imported base's vtable layout. Each slot is seeded
            // with the base's method (typically an abstract-method shim);
            // every override on this concrete record replaces the
            // corresponding slot below.
            //
            // Slots inherited from another module are qualified
            // `Module.RecordType_Method` so the JIT vtable patcher can
            // recognise them as cross-module references and resolve the
            // address from the loader's import-symbol map.
            find_imported_module_sema(module_name)
                .map(|sema| {
                    collect_inherited_method_names(
                        base_record_name,
                        &sema.symbols,
                        Some(module_name.as_str()),
                    )
                    .into_iter()
                    .map(|(decl_module, decl_type_name, method_name)| match decl_module {
                        Some(m) => format!("{m}.{decl_type_name}_{method_name}"),
                        None => format!("{decl_type_name}_{method_name}"),
                    })
                    .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        for method in methods {
            let llvm_name = format!("{}_{}", sym.name, method.name);
            if method.signature.is_new {
                // NEW method: extend the vtable.
                vtable.push(llvm_name);
            } else {
                // Override: find the existing slot (from base) and replace.
                // The base may be either local or imported — call
                // method_slot_in_vtable, which handles both.
                let base_slot = if let Some(bn) = base_name.as_deref() {
                    method_slot_in_vtable(bn, &method.name, module_symbols)
                        .map(|slot| slot as usize)
                } else if let Some((module_name, base_record_name)) = cross_module_base.as_ref() {
                    find_imported_module_sema(module_name).and_then(|sema| {
                        method_slot_in_vtable(base_record_name, &method.name, &sema.symbols)
                            .map(|slot| slot as usize)
                    })
                } else {
                    None
                };
                if let Some(slot) = base_slot {
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
    // CP §6.4: TYPE Foo = POINTER TO FooDesc; methods declared on FooDesc
    // are reachable through Foo. Recurse on the pointee so callers can
    // pass either the pointer-alias name or the record-desc name.
    if let Some(SemanticType::Pointer { target, .. }) = sym.declared_type.as_ref() {
        if let SemanticType::Named { name, module: None, .. } = target.as_ref() {
            return method_slot_in_vtable(name, method_name, module_symbols);
        }
        if let SemanticType::Named { name, module: Some(m), .. } = target.as_ref() {
            // Cross-module pointer alias (e.g. `Files.File = POINTER TO Files.FileDesc`
            // when looked at from outside `Files`).
            let sema = find_imported_module_sema(m)?;
            return method_slot_in_vtable(name, method_name, &sema.symbols);
        }
    }
    let SemanticType::Record { base, methods, .. } = sym.declared_type.as_ref()? else {
        return None;
    };

    // Count how many slots the base type has.
    let base_slot_count: u32 = base
        .as_deref()
        .map(|b| match b {
            SemanticType::Named { name, module: None, .. } => {
                count_vtable_slots(name, module_symbols)
            }
            SemanticType::Named { name, module: Some(m), .. } => {
                find_imported_module_sema(m)
                    .map(|sema| count_vtable_slots(name, &sema.symbols))
                    .unwrap_or(0)
            }
            _ => 0,
        })
        .unwrap_or(0);

    // Check if the method is NEW in this type.
    let new_methods: Vec<&newcp_sema::MethodType> =
        methods.iter().filter(|m| m.signature.is_new).collect();
    if let Some(pos) = new_methods.iter().position(|m| m.name == method_name) {
        return Some(base_slot_count + pos as u32);
    }

    // Not NEW here — it's an override; delegate to the base. Cross-
    // module bases are loaded via `find_imported_module_sema`.
    match base.as_deref() {
        Some(SemanticType::Named { name, module: None, .. }) => {
            method_slot_in_vtable(name, method_name, module_symbols)
        }
        Some(SemanticType::Named { name, module: Some(m), .. }) => {
            let sema = find_imported_module_sema(m)?;
            method_slot_in_vtable(name, method_name, &sema.symbols)
        }
        _ => None,
    }
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
    // Pointer-alias passthrough — see method_slot_in_vtable for the
    // motivation. `Foo = POINTER TO FooDesc` -> count slots of FooDesc.
    if let SemanticType::Pointer { target, .. } = ty {
        if let SemanticType::Named { name, module: None, .. } = target.as_ref() {
            return count_vtable_slots(name, module_symbols);
        }
        if let SemanticType::Named { name, module: Some(m), .. } = target.as_ref() {
            return find_imported_module_sema(m)
                .map(|sema| count_vtable_slots(name, &sema.symbols))
                .unwrap_or(0);
        }
    }
    let (base, methods) = match ty {
        SemanticType::Record { base, methods, .. } => (base, methods),
        _ => return 0,
    };
    let base_count: u32 = base
        .as_deref()
        .map(|b| match b {
            SemanticType::Named { name, module: None, .. } => count_vtable_slots(name, module_symbols),
            SemanticType::Named { name, module: Some(m), .. } => {
                find_imported_module_sema(m)
                    .map(|sema| count_vtable_slots(name, &sema.symbols))
                    .unwrap_or(0)
            }
            _ => 0,
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
            let flat = flatten_fields_deep(sem_ty, module_symbols, module_name, import_cache);
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
                let flat = flatten_fields_deep(sem_ty, &sema.symbols, import_name, import_cache);
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
    current_module: &str,
    import_cache: &mut std::collections::HashMap<String, SemanticModule>,
) -> Vec<(String, IrType)> {
    let mut visited = std::collections::HashSet::new();
    flatten_fields_deep_cross_module(ty, module_symbols, current_module, import_cache, &mut visited)
}

/// Like `flatten_fields_deep` but handles cross-module base types (e.g. RECORD(TypeExt.Animal)).
fn flatten_fields_deep_cross_module(
    ty: &SemanticType,
    module_symbols: &[SemanticSymbol],
    current_module: &str,
    import_cache: &mut std::collections::HashMap<String, SemanticModule>,
    visited: &mut std::collections::HashSet<(String, String)>,
) -> Vec<(String, IrType)> {
    let (base, fields) = match ty {
        SemanticType::Record { base, fields, .. } => (base.as_deref(), fields.as_slice()),
        SemanticType::Named { name, module: None, .. } => {
            let key = (current_module.to_string(), name.clone());
            if !visited.insert(key.clone()) {
                return Vec::new();
            }
            if let Some(sym) = module_symbols.iter().find(|s| s.name == *name) {
                if let Some(resolved) = &sym.declared_type {
                    let resolved_fields = flatten_fields_deep_cross_module(
                        resolved,
                        module_symbols,
                        current_module,
                        import_cache,
                        visited,
                    );
                    visited.remove(&key);
                    return resolved_fields;
                }
            }
            visited.remove(&key);
            return Vec::new();
        }
        SemanticType::Named { name, module: Some(m), .. } => {
            let key = (m.clone(), name.clone());
            if !visited.insert(key.clone()) {
                return Vec::new();
            }
            // Base type is in another module — load and recurse.
            if let Some(base_sema) = load_cached_import(m.as_str(), import_cache) {
                if let Some(sym) = base_sema.symbols.iter().find(|s| s.name == *name) {
                    if let Some(resolved) = &sym.declared_type {
                        // Clone to avoid borrow conflict after cache lookup
                        let resolved = resolved.clone();
                        let symbols: Vec<_> = base_sema.symbols.clone();
                        let m_str = m.clone();
                        let _ = base_sema;
                        let resolved_fields = flatten_fields_deep_cross_module(
                            &resolved,
                            &symbols,
                            &m_str,
                            import_cache,
                            visited,
                        );
                        visited.remove(&key);
                        return resolved_fields;
                    }
                }
            }
            visited.remove(&key);
            return Vec::new();
        }
        _ => return Vec::new(),
    };
    let mut result = Vec::new();
    if let Some(parent) = base {
        result.extend(flatten_fields_deep_cross_module(
            parent,
            module_symbols,
            current_module,
            import_cache,
            visited,
        ));
    }
    for field in fields {
        let ir_ty = map_semantic_type(&field.ty);
        for name in &field.names {
            result.push((name.clone(), ir_ty.clone()));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::collect_named_types;
    use newcp_sema::{NamedTypeKind, SemanticModule, SemanticSymbol, SemanticType, SymbolKind};

    fn type_symbol(name: &str, exported: bool, declared_type: SemanticType) -> SemanticSymbol {
        SemanticSymbol {
            name: name.to_string(),
            kind: SymbolKind::Type,
            exported,
            read_only_export: false,
            declared_type: Some(declared_type),
            const_value: None,
            simd_shape: None,
            param_mode: None,
        }
    }

    #[test]
    fn collect_named_types_breaks_cross_module_named_cycles() {
        let a_symbols = vec![type_symbol(
            "ARec",
            true,
            SemanticType::Named {
                module: Some("B".to_string()),
                name: "BRec".to_string(),
                kind: NamedTypeKind::Imported,
            },
        )];
        let b_symbols = vec![type_symbol(
            "BRec",
            true,
            SemanticType::Named {
                module: Some("A".to_string()),
                name: "ARec".to_string(),
                kind: NamedTypeKind::Imported,
            },
        )];

        let mut import_cache = std::collections::HashMap::new();
        import_cache.insert(
            "A".to_string(),
            SemanticModule {
                name: "A".to_string(),
                imports: vec!["B".to_string()],
                symbols: a_symbols.clone(),
                procedures: Vec::new(),
                selector_resolutions: Vec::new(),
                diagnostics: Vec::new(),
            },
        );
        import_cache.insert(
            "B".to_string(),
            SemanticModule {
                name: "B".to_string(),
                imports: vec!["A".to_string()],
                symbols: b_symbols,
                procedures: Vec::new(),
                selector_resolutions: Vec::new(),
                diagnostics: Vec::new(),
            },
        );

        let named_types = collect_named_types("A", &["B".to_string()], &a_symbols, &mut import_cache);

        assert!(
            !named_types.contains_key("B.BRec"),
            "expected cyclic imported named type to be skipped rather than recurse forever"
        );
    }
}
