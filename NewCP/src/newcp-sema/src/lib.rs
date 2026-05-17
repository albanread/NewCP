use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use newcp_parser::{
    read_module_ast, BinaryOp, Declaration, Designator, ExportMark, Expr, FPSection, FieldDecl,
    FormalParameters, Guard, Literal, MethodFlavor, ModuleAst, ParamMode, ProcedureBody,
    ProcedureDecl, QualIdent, RecordFlavor, Selector, Statement, SysFlag, TypeDecl, TypeExpr,
    UnaryOp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Import,
    Constant,
    Type,
    Variable,
    Procedure,
    Parameter,
    Receiver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    AnyPtr,
    AnyRec,
    Boolean,
    Byte,
    Char,
    IntShort,
    Integer,
    LongInt,
    Real,
    Set,
    ShortChar,
    ShortInt,
    ShortReal,
    String,
    ShortString,
}

impl BuiltinType {
    fn name(self) -> &'static str {
        match self {
            Self::AnyPtr => "ANYPTR",
            Self::AnyRec => "ANYREC",
            Self::Boolean => "BOOLEAN",
            Self::Byte => "BYTE",
            Self::Char => "CHAR",
            Self::IntShort => "INTSHORT",
            Self::Integer => "INTEGER",
            Self::LongInt => "LONGINT",
            Self::Real => "REAL",
            Self::Set => "SET",
            Self::String => "String",
            Self::ShortChar => "SHORTCHAR",
            Self::ShortInt => "SHORTINT",
            Self::ShortReal => "SHORTREAL",
            Self::ShortString => "Shortstring",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinProc {
    Abs,
    Ash,
    Assert,
    Bits,
    Cap,
    Chr,
    Dec,
    Entier,
    Excl,
    Halt,
    Inc,
    Incl,
    Len,
    Long,
    Max,
    Min,
    New,
    Odd,
    Ord,
    Short,
    Size,
    SystemAdr,
    SystemVal,
    SystemLsh,
    SystemRot,
    SystemTyp,
    SystemBit,
    SystemGet,
    SystemPut,
    SystemMove,
    SystemNew,
    SystemGetReg,
    SystemPutReg,
}

impl BuiltinProc {
    fn name(self) -> &'static str {
        match self {
            Self::Abs => "ABS",
            Self::Ash => "ASH",
            Self::Assert => "ASSERT",
            Self::Bits => "BITS",
            Self::Cap => "CAP",
            Self::Chr => "CHR",
            Self::Dec => "DEC",
            Self::Entier => "ENTIER",
            Self::Excl => "EXCL",
            Self::Halt => "HALT",
            Self::Inc => "INC",
            Self::Incl => "INCL",
            Self::Len => "LEN",
            Self::Long => "LONG",
            Self::Max => "MAX",
            Self::Min => "MIN",
            Self::New => "NEW",
            Self::Odd => "ODD",
            Self::Ord => "ORD",
            Self::Short => "SHORT",
            Self::Size => "SIZE",
            Self::SystemAdr => "ADR",
            Self::SystemVal => "VAL",
            Self::SystemLsh => "LSH",
            Self::SystemRot => "ROT",
            Self::SystemTyp => "TYP",
            Self::SystemBit => "BIT",
            Self::SystemGet => "GET",
            Self::SystemPut => "PUT",
            Self::SystemMove => "MOVE",
            Self::SystemNew => "NEW",
            Self::SystemGetReg => "GETREG",
            Self::SystemPutReg => "PUTREG",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedTypeKind {
    UserDefined,
    Imported,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldType {
    pub names: Vec<String>,
    pub ty: SemanticType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodType {
    pub name: String,
    pub signature: ProcedureType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterType {
    pub mode: Option<ParamMode>,
    pub names: Vec<String>,
    pub ty: SemanticType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureType {
    pub receiver: Option<Box<SemanticType>>,
    pub parameters: Vec<ParameterType>,
    pub result_type: Option<Box<SemanticType>>,
    pub is_new: bool,
    pub flavor: Option<MethodFlavor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordLayout {
    Tagged,
    Untagged,
    UntaggedNoAlign,
    UntaggedAlign2,
    UntaggedAlign8,
    Union,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NormalizedSysFlag {
    Untagged,
    NoAlign,
    Align2,
    Align8,
    Union,
    Nil,
    CCall,
    Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdLaneKind {
    Float32,
    Float64,
    Int32,
    Int64,
}

impl SimdLaneKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Float32 => "f32",
            Self::Float64 => "f64",
            Self::Int32 => "i32",
            Self::Int64 => "i64",
        }
    }

    fn lane_bytes(self) -> usize {
        match self {
            Self::Float32 | Self::Int32 => 4,
            Self::Float64 | Self::Int64 => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdLayout {
    PackedRecord,
    ScalarArray,
    ArrayOfStruct,
}

impl SimdLayout {
    fn as_str(self) -> &'static str {
        match self {
            Self::PackedRecord => "packed-record",
            Self::ScalarArray => "scalar-array",
            Self::ArrayOfStruct => "array-of-struct",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimdShape {
    pub layout: SimdLayout,
    pub lane_kind: SimdLaneKind,
    pub lane_count: usize,
    pub packed_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecordMember {
    Field(SemanticType),
    Method(ProcedureType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Builtin(BuiltinType),
    BuiltinProc(BuiltinProc),
    Nil,
    Named {
        module: Option<String>,
        name: String,
        kind: NamedTypeKind,
    },
    Array {
        lengths: Vec<String>,
        element_type: Box<SemanticType>,
        untagged: bool,
    },
    Record {
        flavor: Option<RecordFlavor>,
        layout: RecordLayout,
        base: Option<Box<SemanticType>>,
        fields: Vec<FieldType>,
        methods: Vec<MethodType>,
    },
    Pointer {
        target: Box<SemanticType>,
        untagged: bool,
    },
    Procedure(ProcedureType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub exported: bool,
    /// True when this symbol was exported with a '-' mark (read-only for external modules).
    pub read_only_export: bool,
    pub declared_type: Option<SemanticType>,
    pub const_value: Option<ConstValue>,
    pub simd_shape: Option<SimdShape>,
    /// Only meaningful for `kind == Parameter`: the parameter's CP
    /// passing mode (`Some(In)` / `Some(Var)` / `Some(Out)`, or `None`
    /// for value-mode). Used by assignment validation to reject writes
    /// through `IN` parameters.
    pub param_mode: Option<ParamMode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Integer(i128),
    Real(f64),
    String(String),
    Char(char),
    Boolean(bool),
    /// CP SET constant — the resolved bitmask. SET literals fold
    /// at sema time so `IN`/`+`/`*`/`-` against a CONST SET use the
    /// real bits rather than zero (the value a missing const_value
    /// would inherit from an uninitialised global).
    Set(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorResolutionKind {
    TypeGuard,
    ProcedureCall,
    Unresolved,
}

impl SelectorResolutionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::TypeGuard => "type-guard",
            Self::ProcedureCall => "call",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorResolution {
    pub procedure: Option<String>,
    pub designator: String,
    pub selector: String,
    pub kind: SelectorResolutionKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub procedure: Option<String>,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticProcedure {
    pub name: String,
    /// `Some("Outer")` when this procedure is nested inside `Outer`.
    pub parent_proc: Option<String>,
    pub exported: bool,
    pub signature: ProcedureType,
    pub local_symbols: Vec<SemanticSymbol>,
    /// Outer-scope variables captured by this nested procedure, in the order
    /// they are prepended as implicit pointer parameters at the call site.
    pub upvalues: Vec<(String, SemanticType)>,
    pub selector_resolutions: Vec<SelectorResolution>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticModule {
    pub name: String,
    pub imports: Vec<String>,
    pub symbols: Vec<SemanticSymbol>,
    pub procedures: Vec<SemanticProcedure>,
    pub selector_resolutions: Vec<SelectorResolution>,
    pub diagnostics: Vec<SemanticDiagnostic>,
    /// Symbols of every module the analyser pulled in transitively
    /// (this module's direct imports plus *their* imports).  Exposed
    /// so a parent analyser can fold them into its own
    /// `imported_modules` table — necessary when an alias chain
    /// crosses several modules (e.g. TextViews uses HostStores
    /// which exposes a Stores.Store return type, requiring TextViews
    /// to see Stores' symbol table even though it doesn't import
    /// Stores directly).
    pub imported_modules: HashMap<String, Vec<SemanticSymbol>>,
}

pub fn dump_sema(path: &Path) -> String {
    match analyze_module(path) {
        Ok(module) => render_module_dump(path, &module),
        Err(error) => format!("newcp-sema error\ninput: {}\nerror: {}", path.display(), error),
    }
}

/// Returns a compact diagnostics report for `path`.
///
/// Format when clean:
/// ```text
/// check: <path>
/// ok
/// ```
///
/// Format when errors exist:
/// ```text
/// check: <path>
/// error  <procedure-or-module>@<line>:<col>: <message>
/// error  ...
/// ```
///
/// Parse errors are reported as a single `parse-error` line.
pub fn check_module(path: &Path) -> String {
    let mut lines = vec![format!("check: {}", path.display())];
    match analyze_module(path) {
        Ok(module) => {
            let mut all_diags: Vec<String> = module
                .diagnostics
                .iter()
                .map(|d| format_diagnostic("<module>", d))
                .collect();
            for proc in &module.procedures {
                for d in &proc.diagnostics {
                    all_diags.push(format_diagnostic(&proc.name, d));
                }
            }
            if all_diags.is_empty() {
                lines.push("ok".to_string());
            } else {
                lines.extend(all_diags);
            }
        }
        Err(parse_error) => {
            lines.push(format!("parse-error: {parse_error}"));
        }
    }
    lines.join("\n")
}

fn format_diagnostic(scope: &str, d: &SemanticDiagnostic) -> String {
    let scope_label = d.procedure.as_deref().unwrap_or(scope);
    format!("error  {}@{}:{}: {}", scope_label, d.line, d.column, d.message)
}

pub fn analyze_module(path: &Path) -> Result<SemanticModule, String> {
    let module = read_module_ast(path)?;
    Ok(analyze_module_ast_with_source_dir(&module, path.parent()))
}

pub fn analyze_module_ast(module: &ModuleAst) -> SemanticModule {
    analyze_module_ast_with_source_dir(module, None)
}

/// Like `analyze_module_ast`, but uses `source_dir` (typically the
/// directory containing the module's `.cp` file) as the first place to
/// search for sibling-imported modules' sources, before falling back to
/// the cwd-relative `Mod/<X>.cp` lookup.
///
/// Required for fixtures and consumers that don't sit in the workspace's
/// top-level `Mod/` directory (e.g. tests in `Mod/Tests/`).
pub fn analyze_module_ast_with_source_dir(
    module: &ModuleAst,
    source_dir: Option<&Path>,
) -> SemanticModule {
    let mut analyzer = Analyzer::with_source_dir(module, source_dir);
    analyzer.analyze()
}

struct Analyzer<'a> {
    module: &'a ModuleAst,
    has_system_import: bool,
    module_type_names: HashSet<String>,
    module_symbols: Vec<SemanticSymbol>,
    /// `import_name -> top-level symbols of that module`. Populated by
    /// recursively analysing each imported `Mod/<X>.cp` source so that
    /// cross-module type-alias resolution, record-base walks, override
    /// detection, and subtype assignability all see the imported module's
    /// definitions. Empty on read failure (the import simply behaves
    /// opaquely, which preserves the prior "no cross-module sema"
    /// behaviour for missing-source cases).
    imported_modules: HashMap<String, Vec<SemanticSymbol>>,
    procedures: Vec<SemanticProcedure>,
    selector_resolutions: Vec<SelectorResolution>,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> Analyzer<'a> {
    fn new(module: &'a ModuleAst) -> Self {
        Self::with_source_dir(module, None)
    }

    fn with_source_dir(module: &'a ModuleAst, source_dir: Option<&Path>) -> Self {
        let imported_modules = load_imported_module_symbols(module, source_dir);
        Self {
            module,
            has_system_import: module.imports.iter().any(|item| item.name == "SYSTEM"),
            module_type_names: builtin_type_names(),
            module_symbols: builtin_symbols(),
            imported_modules,
            procedures: Vec::new(),
            selector_resolutions: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn analyze(&mut self) -> SemanticModule {
        self.collect_module_symbols();
        self.validate_module_method_contracts();
        let mut pending_diagnostics = Vec::new();
        self.validate_forward_declarations(
            &self.module.declarations,
            &self.module_type_names,
            None,
            &mut pending_diagnostics,
        );
        self.diagnostics.extend(pending_diagnostics);

        for declaration in &self.module.declarations {
            if let Declaration::Procedure(procedure) = declaration {
                let analyzed = self.analyze_procedure(procedure);
                self.procedures.push(analyzed);
            }
        }

        let empty_locals = Vec::new();
        let mut module_resolutions = Vec::new();
        let mut module_diagnostics = Vec::new();
        if let Some(statements) = &self.module.body {
            self.walk_statements(
                statements,
                None,
                &self.module_type_names,
                &empty_locals,
                None,
                &mut module_resolutions,
                &mut module_diagnostics,
            );
        }
        if let Some(statements) = &self.module.close {
            self.walk_statements(
                statements,
                None,
                &self.module_type_names,
                &empty_locals,
                None,
                &mut module_resolutions,
                &mut module_diagnostics,
            );
        }
        self.selector_resolutions.extend(module_resolutions);
        self.diagnostics.extend(module_diagnostics);

        SemanticModule {
            name: self.module.name.clone(),
            imports: self.module.imports.iter().map(|item| item.name.clone()).collect(),
            symbols: self.module_symbols.clone(),
            procedures: self.procedures.clone(),
            selector_resolutions: self.selector_resolutions.clone(),
            diagnostics: self.diagnostics.clone(),
            imported_modules: self.imported_modules.clone(),
        }
    }

    fn collect_module_symbols(&mut self) {
        let mut scope_names = HashSet::new();
        let mut forward_procedure_names = HashSet::new();
        let mut implemented_procedure_names = HashSet::new();

        for declaration in &self.module.declarations {
            if let Declaration::Type(item) = declaration {
                self.module_type_names.insert(item.name.name.clone());
            }
        }

        for import in &self.module.imports {
            Self::record_duplicate_name(
                &mut scope_names,
                &import.alias.clone().unwrap_or_else(|| import.name.clone()),
                import.span.start.line,
                import.span.start.column,
                None,
                "duplicate module-scope declaration",
                &mut self.diagnostics,
            );
            self.module_symbols.push(SemanticSymbol {
                name: import.alias.clone().unwrap_or_else(|| import.name.clone()),
                kind: SymbolKind::Import,
                exported: false,
                read_only_export: false,
                declared_type: None,
                const_value: None,
                simd_shape: None,
                param_mode: None,
            });
        }

        for declaration in &self.module.declarations {
            match declaration {
                Declaration::Const(item) => {
                    Self::record_identdef_duplicate(&mut scope_names, &item.name, None, "duplicate module-scope declaration", &mut self.diagnostics);
                    let const_value = evaluate_const_expr_with_imports(
                        &item.value,
                        &[],
                        &self.module_symbols,
                        Some(&self.imported_modules),
                    );
                    let declared_type = self
                        .infer_expr_type(&item.value, &[], &self.module_type_names)
                        .or_else(|| const_value_type(&const_value));
                    self.module_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Constant,
                        exported: item.name.export.is_some(),
                        read_only_export: item.name.export == Some(ExportMark::ReadOnly),
                        declared_type,
                        const_value,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
                Declaration::Type(item) => {
                    Self::record_identdef_duplicate(&mut scope_names, &item.name, None, "duplicate module-scope declaration", &mut self.diagnostics);
                    Self::validate_type_expr(&item.ty, &self.module_type_names, self.has_system_import, None, &mut self.diagnostics);
                    // CP §6.3: a user TYPE shadows any builtin pseudo-type
                    // with the same casing.  This matters for "String" /
                    // "Shortstring" (the internal labels for multi-char
                    // string literals) — when CP code does
                    // `TYPE String = ARRAY N OF CHAR`, downstream alias
                    // chasing through `resolve_named_type_alias` must see
                    // the array, not the builtin.  Remove any pre-seeded
                    // builtin Type symbol with this exact name before
                    // pushing the user's version.
                    self.module_symbols
                        .retain(|sym| !(sym.kind == SymbolKind::Type && sym.name == item.name.name));
                    self.module_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Type,
                        exported: item.name.export.is_some(),
                        read_only_export: item.name.export == Some(ExportMark::ReadOnly),
                        declared_type: Some(self.resolve_type_decl(
                            Some(&item.name.name),
                            &item.ty,
                            &self.module_type_names,
                            &[],
                        )),
                        const_value: None,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
                Declaration::Var(item) => {
                    Self::validate_type_expr(&item.ty, &self.module_type_names, self.has_system_import, None, &mut self.diagnostics);
                    let declared_type = self.resolve_type_expr(&item.ty, &self.module_type_names);
                    for name in &item.names {
                        Self::record_identdef_duplicate(&mut scope_names, name, None, "duplicate module-scope declaration", &mut self.diagnostics);
                        self.module_symbols.push(SemanticSymbol {
                            name: name.name.clone(),
                            kind: SymbolKind::Variable,
                            exported: name.export.is_some(),
                            read_only_export: name.export == Some(ExportMark::ReadOnly),
                            declared_type: Some(declared_type.clone()),
                            const_value: None,
                            simd_shape: None,
                            param_mode: None,
                        });
                    }
                }
                Declaration::Procedure(item) => {
                    Self::record_module_procedure_duplicate(
                        &mut implemented_procedure_names,
                        Some(&forward_procedure_names),
                        &item.heading,
                        "duplicate module-scope declaration",
                        &mut self.diagnostics,
                    );
                    Self::validate_heading_types(&item.heading, &self.module_type_names, self.has_system_import, None, &mut self.diagnostics);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: item.heading.name.export.is_some(),
                        read_only_export: item.heading.name.export == Some(ExportMark::ReadOnly),
                        declared_type: Some(SemanticType::Procedure(
                            self.resolve_procedure_signature(&self.module_type_names, item),
                        )),
                        const_value: None,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
                Declaration::Forward(item) => {
                    Self::record_module_procedure_duplicate(
                        &mut forward_procedure_names,
                        Some(&implemented_procedure_names),
                        &item.heading,
                        "duplicate module-scope declaration",
                        &mut self.diagnostics,
                    );
                    Self::validate_heading_types(&item.heading, &self.module_type_names, self.has_system_import, None, &mut self.diagnostics);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: item.heading.name.export.is_some(),
                        read_only_export: item.heading.name.export == Some(ExportMark::ReadOnly),
                        declared_type: Some(SemanticType::Procedure(self.resolve_heading_signature(
                            &item.heading,
                            &self.module_type_names,
                        ))),
                        const_value: None,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
            }
        }

        annotate_simd_shapes(&mut self.module_symbols, &[]);
    }

    fn analyze_procedure(&mut self, procedure: &'a ProcedureDecl) -> SemanticProcedure {
        let mut scope_type_names = self.module_type_names.clone();
        let mut local_symbols = Vec::new();
        let mut selector_resolutions = Vec::new();
        let mut diagnostics = Vec::new();
        let mut scope_names = HashSet::new();

        let signature = self.resolve_procedure_signature(&scope_type_names, procedure);

        if let Some(receiver) = &procedure.heading.receiver {
            Self::record_duplicate_name(
                &mut scope_names,
                &receiver.name,
                receiver.span.start.line,
                receiver.span.start.column,
                Some(&procedure.heading.name.name),
                "duplicate procedure-scope declaration",
                &mut diagnostics,
            );
            Self::validate_receiver_type(
                receiver.ty.as_str(),
                receiver.span.start.line,
                receiver.span.start.column,
                &scope_type_names,
                Some(&procedure.heading.name.name),
                &mut diagnostics,
            );
            local_symbols.push(SemanticSymbol {
                name: receiver.name.clone(),
                kind: SymbolKind::Receiver,
                exported: false,
                read_only_export: false,
                // Symbol carries the AS-WRITTEN receiver type
                // (e.g. `Foo` when declared `(a: Foo)` where
                // `Foo = POINTER TO FooDesc`).  Procedure
                // signature.receiver below uses the canonical
                // form via `resolve_procedure_signature` —
                // method-dispatch / IR matching keep working.
                declared_type: Some(self.resolve_receiver_symbol_type(receiver.ty.as_str(), &scope_type_names)),
                const_value: None,
                simd_shape: None,
                param_mode: None,
            });
        }

        if let Some(parameters) = &procedure.heading.formal_parameters {
            self.collect_parameter_symbols(
                parameters,
                &mut local_symbols,
                &mut scope_type_names,
                &mut scope_names,
                Some(&procedure.heading.name.name),
                &mut diagnostics,
            );
        }

        if let Some(body) = &procedure.body {
            self.collect_local_declaration_symbols(
                body,
                &mut local_symbols,
                &mut scope_type_names,
                &mut scope_names,
                Some(&procedure.heading.name.name),
                &mut diagnostics,
            );
            self.validate_forward_declarations(
                &body.declarations,
                &scope_type_names,
                Some(&procedure.heading.name.name),
                &mut diagnostics,
            );
            if let Some(statements) = &body.body {
                self.walk_statements(
                    statements,
                    Some(&procedure.heading.name.name),
                    &scope_type_names,
                    &local_symbols,
                    signature.result_type.as_deref(),
                    &mut selector_resolutions,
                    &mut diagnostics,
                );
            }
        }

        self.selector_resolutions.extend(selector_resolutions.clone());
        self.diagnostics.extend(diagnostics.clone());
        annotate_simd_shapes(&mut local_symbols, &self.module_symbols);

        let outer_name = procedure.heading.name.name.clone();

        // Recursively analyze nested procedures and register them in self.procedures
        // with a qualified name ("Outer_Inner").  They are emitted as flat functions
        // by the IR lowerer, with captured outer variables passed as implicit Ref params.
        if let Some(body) = &procedure.body {
            for declaration in &body.declarations {
                if let Declaration::Procedure(nested) = declaration {
                    let nested_sema = self.analyze_nested_procedure(
                        nested,
                        &outer_name,
                        &local_symbols,
                        &scope_type_names,
                    );
                    self.procedures.push(nested_sema);
                }
            }
        }

        SemanticProcedure {
            name: outer_name,
            parent_proc: None,
            exported: procedure.heading.name.export.is_some(),
            signature,
            local_symbols,
            upvalues: Vec::new(),
            selector_resolutions,
            diagnostics,
        }
    }

    /// Analyze a procedure nested inside `parent_name`, using `parent_locals`
    /// for outer-scope variable resolution.  Returns a `SemanticProcedure` with:
    /// - `name` = "{parent_name}_{nested_name}" (qualified flat name)
    /// - `parent_proc` = `Some(parent_name.to_string())`
    /// - `upvalues` = outer variables from `parent_locals` that are referenced in the body
    fn analyze_nested_procedure(
        &mut self,
        nested: &'a ProcedureDecl,
        parent_name: &str,
        parent_locals: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
    ) -> SemanticProcedure {
        let qualified_name = format!("{parent_name}_{}", nested.heading.name.name);
        let mut scope_type_names = scope_type_names.clone();
        let mut local_symbols: Vec<SemanticSymbol> = Vec::new();
        let mut scope_names = HashSet::new();
        let mut selector_resolutions = Vec::new();
        let mut diagnostics = Vec::new();

        let signature = self.resolve_procedure_signature(&scope_type_names, nested);

        // Collect the nested proc's own params into local_symbols.
        if let Some(parameters) = &nested.heading.formal_parameters {
            self.collect_parameter_symbols(
                parameters,
                &mut local_symbols,
                &mut scope_type_names,
                &mut scope_names,
                Some(&qualified_name),
                &mut diagnostics,
            );
        }

        // Collect the nested proc's own local declarations.
        if let Some(body) = &nested.body {
            self.collect_local_declaration_symbols(
                body,
                &mut local_symbols,
                &mut scope_type_names,
                &mut scope_names,
                Some(&qualified_name),
                &mut diagnostics,
            );
        }

        // Determine which parent-scope variables are referenced in the nested body.
        let own_names: HashSet<String> = local_symbols.iter().map(|s| s.name.clone()).collect();
        let free_names: HashSet<String> = nested
            .body
            .as_ref()
            .and_then(|b| b.body.as_ref())
            .map(|stmts| collect_free_names_in_stmts(stmts))
            .unwrap_or_default();

        let upvalues: Vec<(String, SemanticType)> = parent_locals
            .iter()
            .filter(|s| {
                // Include the outer method's receiver too — BlackBox-faithful
                // CP code commonly captures `self` from a type-bound method
                // (e.g. `Frame.DrawPath` has a nested `Draw` that reads
                // `f.unit`, `f.gx`, etc.).  Without this, the nested proc's
                // body sees `f` as unresolved and IR falls through to
                // `Opaque`, causing pointer→i64 cast failures downstream.
                matches!(
                    s.kind,
                    SymbolKind::Variable | SymbolKind::Parameter | SymbolKind::Receiver
                )
                    && free_names.contains(&s.name)
                    && !own_names.contains(&s.name)
            })
            .filter_map(|s| s.declared_type.as_ref().map(|ty| (s.name.clone(), ty.clone())))
            .collect();

        // Add upvalue vars to local_symbols so the nested proc's body can resolve them.
        for (name, ty) in &upvalues {
            if !own_names.contains(name) {
                local_symbols.push(SemanticSymbol {
                    name: name.clone(),
                    kind: SymbolKind::Variable,
                    exported: false,
                    read_only_export: false,
                    declared_type: Some(ty.clone()),
                    const_value: None,
                    simd_shape: None,
                    param_mode: None,
                });
            }
        }

        // Build a combined symbol table for walking the nested body.
        // Parent locals come first so that own params/locals — appended
        // last — appear last in the vector. `lookup_symbol` iterates in
        // reverse, so the nested proc's own bindings are found first and
        // correctly shadow same-named parent params (e.g. nested
        // `Byte(x: INTEGER)` inside `Externalize(.., x: Integer)`).
        let combined_symbols: Vec<SemanticSymbol> = parent_locals
            .iter()
            .cloned()
            .chain(local_symbols.iter().cloned())
            .collect();

        if let Some(body) = &nested.body {
            if let Some(statements) = &body.body {
                self.walk_statements(
                    statements,
                    Some(&qualified_name),
                    &scope_type_names,
                    &combined_symbols,
                    signature.result_type.as_deref(),
                    &mut selector_resolutions,
                    &mut diagnostics,
                );
            }
        }

        self.selector_resolutions.extend(selector_resolutions.clone());
        self.diagnostics.extend(diagnostics.clone());

        SemanticProcedure {
            name: qualified_name,
            parent_proc: Some(parent_name.to_string()),
            exported: false,
            signature,
            local_symbols,
            upvalues,
            selector_resolutions,
            diagnostics,
        }
    }

    fn validate_module_method_contracts(&mut self) {
        for declaration in &self.module.declarations {
            let Declaration::Procedure(procedure) = declaration else {
                continue;
            };
            let Some(receiver) = &procedure.heading.receiver else {
                continue;
            };

            let Some(record_decl) = self.find_record_decl(&receiver.ty) else {
                continue;
            };
            let record_flavor = match &record_decl.ty {
                TypeExpr::Record { flavor, .. } => flavor.clone(),
                _ => None,
            };
            let base_name = match &record_decl.ty {
                TypeExpr::Record { base, .. } => base.as_ref().map(|item| item.name.clone()),
                _ => None,
            };
            let procedure_name = &procedure.heading.name.name;

            match procedure.heading.attributes.flavor {
                Some(MethodFlavor::Abstract | MethodFlavor::Empty) => {
                    if procedure.body.is_some() {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "{} method {} must not have a procedure body",
                                method_flavor_name(procedure.heading.attributes.flavor.unwrap()),
                                procedure_name
                            ),
                        ));
                    }
                }
                _ => {
                    if procedure.body.is_none() {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!("concrete method {} must have a procedure body", procedure_name),
                        ));
                    }
                }
            }

            if procedure.heading.attributes.flavor == Some(MethodFlavor::Empty) {
                if procedure
                    .heading
                    .formal_parameters
                    .as_ref()
                    .is_some_and(|parameters| parameters.result_type.is_some())
                {
                    self.diagnostics.push(make_diagnostic(
                        None,
                        procedure.heading.span.start.line,
                        procedure.heading.span.start.column,
                        format!("empty method {} must not return a result", procedure_name),
                    ));
                }

                if procedure
                    .heading
                    .formal_parameters
                    .as_ref()
                    .is_some_and(has_out_parameter)
                {
                    self.diagnostics.push(make_diagnostic(
                        None,
                        procedure.heading.span.start.line,
                        procedure.heading.span.start.column,
                        format!("empty method {} must not have OUT parameters", procedure_name),
                    ));
                }
            }

            if procedure.heading.attributes.flavor == Some(MethodFlavor::Abstract)
                && record_decl.name.export.is_some()
                && procedure.heading.name.export.is_none()
            {
                self.diagnostics.push(make_diagnostic(
                    None,
                    procedure.heading.span.start.line,
                    procedure.heading.span.start.column,
                    format!(
                        "abstract method {} of exported record {} must be exported",
                        procedure_name, receiver.ty
                    ),
                ));
            }

            let inherited = self.find_inherited_method(&receiver.ty, procedure_name);
            match inherited {
                Some(base_method) => {
                    if procedure.heading.attributes.is_new {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "redefining method {} must not use NEW",
                                procedure_name
                            ),
                        ));
                    }

                    if base_method.heading.attributes.flavor.is_none() {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "method {} cannot redefine final method declared in {}",
                                procedure_name,
                                base_method
                                    .heading
                                    .receiver
                                    .as_ref()
                                    .map(|item| item.ty.as_str())
                                    .unwrap_or("<unknown>")
                            ),
                        ));
                    }

                    match base_method.heading.name.export {
                        Some(base_export) => {
                            if record_decl.name.export.is_some() && procedure.heading.name.export.is_none() {
                                self.diagnostics.push(make_diagnostic(
                                    None,
                                    procedure.heading.span.start.line,
                                    procedure.heading.span.start.column,
                                    format!(
                                        "overriding method {} of exported record {} must be exported",
                                        procedure_name, receiver.ty
                                    ),
                                ));
                            }

                            if let Some(actual_export) = procedure.heading.name.export
                                && actual_export != base_export
                            {
                                self.diagnostics.push(make_diagnostic(
                                    None,
                                    procedure.heading.span.start.line,
                                    procedure.heading.span.start.column,
                                    format!(
                                        "overriding method {} must use the same export mark as the base method ({})",
                                        procedure_name,
                                        export_mark_name(base_export)
                                    ),
                                ));
                            }
                        }
                        None => {
                            if procedure.heading.name.export.is_some() {
                                self.diagnostics.push(make_diagnostic(
                                    None,
                                    procedure.heading.span.start.line,
                                    procedure.heading.span.start.column,
                                    format!(
                                        "overriding method {} must not be exported because the base method is not exported",
                                        procedure_name
                                    ),
                                ));
                            }
                        }
                    }

                    let expected = self.resolve_procedure_signature(&self.module_type_names, base_method);
                    let actual = self.resolve_procedure_signature(&self.module_type_names, procedure);
                    if !self.method_signatures_match(&expected, &actual) {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "method {} does not match overridden signature",
                                procedure_name
                            ),
                        ));
                    }

                    if procedure.heading.attributes.flavor == Some(MethodFlavor::Abstract)
                        && base_method.heading.attributes.flavor != Some(MethodFlavor::Abstract)
                    {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "abstract method {} may only redefine an abstract method",
                                procedure_name
                            ),
                        ));
                    }

                    if procedure.heading.attributes.flavor == Some(MethodFlavor::Empty)
                        && !matches!(
                            base_method.heading.attributes.flavor,
                            Some(MethodFlavor::Empty | MethodFlavor::Abstract)
                        )
                    {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "empty method {} may only redefine an empty or abstract method",
                                procedure_name
                            ),
                        ));
                    }
                }
                None => {
                    // Local lookup found nothing. Before flagging "must
                    // use NEW", check whether the method is inherited
                    // from an *imported* base — if so, this is a
                    // cross-module override and should not require NEW.
                    let canonical_recv = self.canonical_receiver_record(&receiver.ty);
                    let cross_module_override = self
                        .has_inherited_method_anywhere(&canonical_recv, procedure_name);
                    if !procedure.heading.attributes.is_new && !cross_module_override {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "newly introduced method {} must use NEW",
                                procedure_name
                            ),
                        ));
                    }
                    // The reverse: declares NEW but actually overrides
                    // an inherited cross-module method — flag it. This
                    // mirrors the local-base check on the Some branch.
                    if procedure.heading.attributes.is_new && cross_module_override {
                        self.diagnostics.push(make_diagnostic(
                            None,
                            procedure.heading.span.start.line,
                            procedure.heading.span.start.column,
                            format!(
                                "redefining method {} must not use NEW",
                                procedure_name
                            ),
                        ));
                    }
                }
            }

            if procedure.heading.attributes.flavor == Some(MethodFlavor::Abstract)
                && record_flavor != Some(RecordFlavor::Abstract)
            {
                self.diagnostics.push(make_diagnostic(
                    None,
                    procedure.heading.span.start.line,
                    procedure.heading.span.start.column,
                    format!(
                        "record {} must be ABSTRACT to declare abstract method {}",
                        receiver.ty, procedure_name
                    ),
                ));
            }

            if procedure.heading.attributes.flavor == Some(MethodFlavor::Extensible)
                && !matches!(record_flavor, Some(RecordFlavor::Extensible | RecordFlavor::Abstract))
            {
                self.diagnostics.push(make_diagnostic(
                    None,
                    procedure.heading.span.start.line,
                    procedure.heading.span.start.column,
                    format!(
                        "record {} must be EXTENSIBLE or ABSTRACT to declare extensible method {}",
                        receiver.ty, procedure_name
                    ),
                ));
            }

            if procedure.heading.attributes.is_new
                && procedure.heading.attributes.flavor == Some(MethodFlavor::Empty)
                && !matches!(record_flavor, Some(RecordFlavor::Extensible | RecordFlavor::Abstract))
            {
                self.diagnostics.push(make_diagnostic(
                    None,
                    procedure.heading.span.start.line,
                    procedure.heading.span.start.column,
                    format!(
                        "record {} must be EXTENSIBLE or ABSTRACT to declare new empty method {}",
                        receiver.ty, procedure_name
                    ),
                ));
            }

            if record_flavor == Some(RecordFlavor::Abstract)
                && let Some(base_name) = base_name.as_deref()
            {
                // Resolve the base's flavor.  Local types use the
                // current module's declarations; cross-module bases
                // (`Models.ModelDesc extends Stores.StoreDesc`) need
                // an import lookup since `record_type_info` only sees
                // the local module.  Canonicalize the receiver name
                // so the pointer-alias receiver form (`(m: Model)`)
                // queries the underlying record (`ModelDesc`).
                let canonical_recv = self.canonical_receiver_record(&receiver.ty);
                let qualified_info = self.record_type_info_qualified(&canonical_recv);
                let cross_mod_base = qualified_info
                    .as_ref()
                    .and_then(|(_, b)| b.as_ref())
                    .and_then(|(m, n)| m.as_ref().map(|module| (module.clone(), n.clone())));
                let base_is_abstract = if let Some((module, name)) = cross_mod_base {
                    self.imported_modules
                        .get(&module)
                        .and_then(|symbols| symbols.iter().find(|s| s.name == name))
                        .and_then(|s| s.declared_type.as_ref())
                        .map(|t| matches!(t, SemanticType::Record { flavor: Some(RecordFlavor::Abstract), .. }))
                        .unwrap_or(false)
                } else {
                    self.record_type_info(base_name)
                        .map(|(flavor, _)| flavor)
                        == Some(Some(RecordFlavor::Abstract))
                };
                if !base_is_abstract {
                    self.diagnostics.push(make_diagnostic(
                        None,
                        procedure.heading.span.start.line,
                        procedure.heading.span.start.column,
                        format!(
                            "abstract record {} must extend an abstract base type",
                            receiver.ty
                        ),
                    ));
                }
            }
        }

        for declaration in &self.module.declarations {
            let Declaration::Type(type_decl) = declaration else {
                continue;
            };
            let Some((flavor, base_name)) = self.record_type_info(&type_decl.name.name) else {
                continue;
            };
            if flavor == Some(RecordFlavor::Abstract) {
                continue;
            }
            if base_name.is_none() {
                continue;
            }

            for method in self.effective_methods_for_type(&type_decl.name.name) {
                if method.heading.attributes.flavor == Some(MethodFlavor::Abstract) {
                    self.diagnostics.push(make_diagnostic(
                        None,
                        type_decl.span.start.line,
                        type_decl.span.start.column,
                        format!(
                            "concrete record {} must implement abstract method {}",
                            type_decl.name.name,
                            method.heading.name.name
                        ),
                    ));
                }
            }
        }
    }

    fn validate_forward_declarations(
        &self,
        declarations: &[Declaration],
        scope_type_names: &HashSet<String>,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        for declaration in declarations {
            let Declaration::Forward(forward) = declaration else {
                continue;
            };

            let expected = self.resolve_heading_signature(&forward.heading, scope_type_names);
            let forward_identity = module_procedure_identity(&forward.heading);
            let implementation = declarations.iter().find_map(|item| match item {
                Declaration::Procedure(procedure)
                    if module_procedure_identity(&procedure.heading) == forward_identity =>
                {
                    Some(procedure)
                }
                _ => None,
            });

            match implementation {
                Some(procedure) => {
                    let actual = self.resolve_procedure_signature(scope_type_names, procedure);
                    if actual != expected {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            forward.heading.span.start.line,
                            forward.heading.span.start.column,
                            format!(
                                "forward declaration for {} does not match implementation",
                                forward.heading.name.name
                            ),
                        ));
                    }
                }
                None => diagnostics.push(make_diagnostic(
                    procedure_name,
                    forward.heading.span.start.line,
                    forward.heading.span.start.column,
                    format!(
                        "forward declaration for {} has no matching implementation",
                        forward.heading.name.name
                    ),
                )),
            }
        }
    }

    fn collect_parameter_symbols(
        &self,
        parameters: &FormalParameters,
        local_symbols: &mut Vec<SemanticSymbol>,
        scope_type_names: &mut HashSet<String>,
        scope_names: &mut HashSet<String>,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        for section in &parameters.sections {
            self.collect_parameter_section_symbols(
                section,
                local_symbols,
                scope_type_names,
                scope_names,
                procedure_name,
                diagnostics,
            );
        }
    }

    fn collect_parameter_section_symbols(
        &self,
        section: &FPSection,
        local_symbols: &mut Vec<SemanticSymbol>,
        scope_type_names: &mut HashSet<String>,
        scope_names: &mut HashSet<String>,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        self.collect_type_names(&section.ty, scope_type_names);
        Self::validate_type_expr(&section.ty, scope_type_names, self.has_system_import, procedure_name, diagnostics);
        let declared_type = self.resolve_type_expr(&section.ty, scope_type_names);

        // Value-mode record / fixed-array / open-array parameters are
        // honoured per CP §8.1: the LLVM ABI passes a pointer to the
        // caller's data and the callee prologue (in newcp-llvm's
        // `bind_proc_slots`) memmoves the bytes into a stack-local
        // copy.  No sema rejection here — the codegen handles the
        // private-copy semantics.

        for name in &section.names {
            Self::record_duplicate_name(
                scope_names,
                name,
                section.span.start.line,
                section.span.start.column,
                procedure_name,
                "duplicate procedure-scope declaration",
                diagnostics,
            );
            local_symbols.push(SemanticSymbol {
                name: name.clone(),
                kind: SymbolKind::Parameter,
                exported: false,
                read_only_export: false,
                declared_type: Some(declared_type.clone()),
                const_value: None,
                simd_shape: None,
                param_mode: section.mode,
            });
        }
    }

    /// True if `ty` is (or resolves through Named aliases to) a Record.
    /// Pointer-aliased types do NOT count — they're scalars at the ABI
    /// level. Used by parameter-declaration validation.
    fn semantic_type_is_record(
        &self,
        ty: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        match self.resolve_named_type_one_level(ty, local_symbols) {
            SemanticType::Record { .. } => true,
            SemanticType::Pointer { .. } => false,
            _ => false,
        }
    }

    fn collect_local_declaration_symbols(
        &self,
        body: &ProcedureBody,
        local_symbols: &mut Vec<SemanticSymbol>,
        scope_type_names: &mut HashSet<String>,
        scope_names: &mut HashSet<String>,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        for declaration in &body.declarations {
            if let Declaration::Type(item) = declaration {
                scope_type_names.insert(item.name.name.clone());
            }
        }

        for declaration in &body.declarations {
            match declaration {
                Declaration::Const(item) => {
                    Self::record_identdef_duplicate(scope_names, &item.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    let const_value = evaluate_const_expr_with_imports(
                        &item.value,
                        local_symbols,
                        &self.module_symbols,
                        Some(&self.imported_modules),
                    );
                    let declared_type = self
                        .infer_expr_type(&item.value, local_symbols, scope_type_names)
                        .or_else(|| const_value_type(&const_value));
                    local_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Constant,
                        exported: false,
                        read_only_export: false,
                        declared_type,
                        const_value,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
                Declaration::Type(item) => {
                    Self::record_identdef_duplicate(scope_names, &item.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    Self::validate_type_expr(&item.ty, scope_type_names, self.has_system_import, procedure_name, diagnostics);
                    local_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Type,
                        exported: false,
                        read_only_export: false,
                        declared_type: Some(self.resolve_type_decl(
                            Some(&item.name.name),
                            &item.ty,
                            scope_type_names,
                            local_symbols,
                        )),
                        const_value: None,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
                Declaration::Var(item) => {
                    Self::validate_type_expr(&item.ty, scope_type_names, self.has_system_import, procedure_name, diagnostics);
                    let declared_type = self.resolve_type_expr_with_locals(&item.ty, scope_type_names, local_symbols);
                    for name in &item.names {
                        Self::record_identdef_duplicate(scope_names, name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                        local_symbols.push(SemanticSymbol {
                            name: name.name.clone(),
                            kind: SymbolKind::Variable,
                            exported: false,
                            read_only_export: false,
                            declared_type: Some(declared_type.clone()),
                            const_value: None,
                            simd_shape: None,
                            param_mode: None,
                        });
                    }
                }
                Declaration::Procedure(item) => {
                    if item.heading.receiver.is_some() {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            item.heading.span.start.line,
                            item.heading.span.start.column,
                            format!(
                                "method {} must be declared at module scope",
                                item.heading.name.name
                            ),
                        ));
                    }
                    Self::record_identdef_duplicate(scope_names, &item.heading.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    Self::validate_heading_types(&item.heading, scope_type_names, self.has_system_import, procedure_name, diagnostics);
                    local_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: false,
                        read_only_export: false,
                        declared_type: Some(SemanticType::Procedure(self.resolve_procedure_signature(
                            scope_type_names,
                            item,
                        ))),
                        const_value: None,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
                Declaration::Forward(item) => {
                    if item.heading.receiver.is_some() {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            item.heading.span.start.line,
                            item.heading.span.start.column,
                            format!(
                                "method {} must be declared at module scope",
                                item.heading.name.name
                            ),
                        ));
                    }
                    Self::record_identdef_duplicate(scope_names, &item.heading.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    Self::validate_heading_types(&item.heading, scope_type_names, self.has_system_import, procedure_name, diagnostics);
                    local_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: false,
                        read_only_export: false,
                        declared_type: Some(SemanticType::Procedure(self.resolve_heading_signature(
                            &item.heading,
                            scope_type_names,
                        ))),
                        const_value: None,
                        simd_shape: None,
                        param_mode: None,
                    })
                }
            }
        }
    }

    fn record_identdef_duplicate(
        scope_names: &mut HashSet<String>,
        ident: &newcp_parser::IdentDef,
        procedure_name: Option<&str>,
        message: &str,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        Self::record_duplicate_name(
            scope_names,
            &ident.name,
            ident.span.start.line,
            ident.span.start.column,
            procedure_name,
            message,
            diagnostics,
        );
    }

    fn record_module_procedure_duplicate(
        scope_names: &mut HashSet<String>,
        _allowed_existing: Option<&HashSet<String>>,
        heading: &newcp_parser::ProcedureHeading,
        message: &str,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let identity = module_procedure_identity(heading);
        if scope_names.contains(&identity) {
            diagnostics.push(make_diagnostic(
                None,
                heading.span.start.line,
                heading.span.start.column,
                format!("{}: {}", message, heading.name.name),
            ));
            return;
        }

        scope_names.insert(identity);
    }

    fn record_duplicate_name(
        scope_names: &mut HashSet<String>,
        name: &str,
        line: usize,
        column: usize,
        procedure_name: Option<&str>,
        message: &str,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        if !scope_names.insert(name.to_string()) {
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("{}: {}", message, name),
            ));
        }
    }

    fn validate_heading_types(
        heading: &newcp_parser::ProcedureHeading,
        scope_type_names: &HashSet<String>,
        has_system_import: bool,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        Self::validate_sys_flag(
            heading.sys_flag.as_ref(),
            has_system_import,
            heading.span.start.line,
            heading.span.start.column,
            procedure_name,
            diagnostics,
        );
        if let Some(receiver) = &heading.receiver {
            Self::validate_receiver_type(
                receiver.ty.as_str(),
                receiver.span.start.line,
                receiver.span.start.column,
                scope_type_names,
                procedure_name,
                diagnostics,
            );
        }
        if let Some(parameters) = &heading.formal_parameters {
            for section in &parameters.sections {
                Self::validate_sys_flag(
                    section.sys_flag.as_ref(),
                    has_system_import,
                    section.span.start.line,
                    section.span.start.column,
                    procedure_name,
                    diagnostics,
                );
                Self::validate_type_expr(&section.ty, scope_type_names, has_system_import, procedure_name, diagnostics);
            }
            if let Some(result) = &parameters.result_type {
                Self::validate_type_expr(result, scope_type_names, has_system_import, procedure_name, diagnostics);
            }
        }
    }

    fn validate_receiver_type(
        receiver_type_name: &str,
        line: usize,
        column: usize,
        scope_type_names: &HashSet<String>,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        if builtin_type_by_name(receiver_type_name).is_none() && !scope_type_names.contains(receiver_type_name) {
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("unknown type {}", receiver_type_name),
            ));
        }
    }

    fn validate_type_expr(
        ty: &TypeExpr,
        scope_type_names: &HashSet<String>,
        has_system_import: bool,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        match ty {
            TypeExpr::QualIdent { span, ident } => {
                if ident.module.is_none()
                    && builtin_type_by_name(&ident.name).is_none()
                    && !scope_type_names.contains(&ident.name)
                {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        span.start.line,
                        span.start.column,
                        format!("unknown type {}", ident.name),
                    ));
                }
            }
            TypeExpr::Array { span, sys_flag, element_type, .. } => {
                Self::validate_sys_flag(sys_flag.as_ref(), has_system_import, span.start.line, span.start.column, procedure_name, diagnostics);
                Self::validate_type_expr(element_type, scope_type_names, has_system_import, procedure_name, diagnostics);
            }
            TypeExpr::Record { span, sys_flag, base, fields, .. } => {
                Self::validate_sys_flag(sys_flag.as_ref(), has_system_import, span.start.line, span.start.column, procedure_name, diagnostics);
                if let Some(base) = base {
                    if base.module.is_none()
                        && builtin_type_by_name(&base.name).is_none()
                        && !scope_type_names.contains(&base.name)
                    {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            base.span.start.line,
                            base.span.start.column,
                            format!("unknown type {}", base.name),
                        ));
                    }
                }
                for field in fields {
                    Self::validate_type_expr(&field.ty, scope_type_names, has_system_import, procedure_name, diagnostics);
                }
            }
            TypeExpr::Pointer { span, sys_flag, target, .. } => {
                Self::validate_sys_flag(sys_flag.as_ref(), has_system_import, span.start.line, span.start.column, procedure_name, diagnostics);
                Self::validate_type_expr(target, scope_type_names, has_system_import, procedure_name, diagnostics);
            }
            TypeExpr::Procedure { span, sys_flag, formal_parameters, .. } => {
                Self::validate_sys_flag(sys_flag.as_ref(), has_system_import, span.start.line, span.start.column, procedure_name, diagnostics);
                if let Some(parameters) = formal_parameters {
                    for section in &parameters.sections {
                        Self::validate_sys_flag(
                            section.sys_flag.as_ref(),
                            has_system_import,
                            section.span.start.line,
                            section.span.start.column,
                            procedure_name,
                            diagnostics,
                        );
                        Self::validate_type_expr(&section.ty, scope_type_names, has_system_import, procedure_name, diagnostics);
                    }
                    if let Some(result) = &parameters.result_type {
                        Self::validate_type_expr(result, scope_type_names, has_system_import, procedure_name, diagnostics);
                    }
                }
            }
        }
    }

    fn validate_sys_flag(
        flag: Option<&SysFlag>,
        has_system_import: bool,
        line: usize,
        column: usize,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let Some(flag) = flag else {
            return;
        };

        if !has_system_import {
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("system flag {} requires IMPORT SYSTEM", render_sys_flag(flag)),
            ));
        }
    }

    fn collect_type_names(&self, ty: &TypeExpr, scope_type_names: &mut HashSet<String>) {
        match ty {
            TypeExpr::QualIdent { ident, .. } => {
                if ident.module.is_none() {
                    scope_type_names.insert(ident.name.clone());
                }
            }
            TypeExpr::Array { element_type, .. } => self.collect_type_names(element_type, scope_type_names),
            TypeExpr::Record { base, fields, .. } => {
                if let Some(base) = base {
                    if base.module.is_none() {
                        scope_type_names.insert(base.name.clone());
                    }
                }
                for field in fields {
                    self.collect_type_names(&field.ty, scope_type_names);
                }
            }
            TypeExpr::Pointer { target, .. } => self.collect_type_names(target, scope_type_names),
            TypeExpr::Procedure { formal_parameters, .. } => {
                if let Some(parameters) = formal_parameters {
                    for section in &parameters.sections {
                        self.collect_type_names(&section.ty, scope_type_names);
                    }
                    if let Some(result) = &parameters.result_type {
                        self.collect_type_names(result, scope_type_names);
                    }
                }
            }
        }
    }

    fn resolve_procedure_signature(
        &self,
        scope_type_names: &HashSet<String>,
        procedure: &ProcedureDecl,
    ) -> ProcedureType {
        self.resolve_heading_signature(&procedure.heading, scope_type_names)
    }

    fn resolve_heading_signature(
        &self,
        heading: &newcp_parser::ProcedureHeading,
        scope_type_names: &HashSet<String>,
    ) -> ProcedureType {
        ProcedureType {
            receiver: heading.receiver.as_ref().map(|receiver| {
                Box::new(self.resolve_receiver_type(receiver.ty.as_str(), scope_type_names))
            }),
            parameters: heading
                .formal_parameters
                .as_ref()
                .map(|parameters| self.resolve_parameter_types(parameters, scope_type_names))
                .unwrap_or_default(),
            result_type: heading
                .formal_parameters
                .as_ref()
                .and_then(|parameters| parameters.result_type.as_ref())
                .map(|ty| Box::new(self.resolve_type_expr(ty, scope_type_names))),
            is_new: heading.attributes.is_new,
            flavor: heading.attributes.flavor,
        }
    }

    fn resolve_parameter_types(
        &self,
        parameters: &FormalParameters,
        scope_type_names: &HashSet<String>,
    ) -> Vec<ParameterType> {
        parameters
            .sections
            .iter()
            .map(|section| ParameterType {
                mode: section.mode,
                names: section.names.clone(),
                ty: self.resolve_type_expr(&section.ty, scope_type_names),
            })
            .collect()
    }

    fn resolve_receiver_type(
        &self,
        receiver_type_name: &str,
        scope_type_names: &HashSet<String>,
    ) -> SemanticType {
        // Canonical Record form — used in the procedure's
        // `signature.receiver` so the IR's receiver-matching
        // (`canonicalize_receiver_name` at IR module-lower
        // time) and the override / method-dispatch lookups
        // see a single shape regardless of whether the
        // receiver was declared via the record name or its
        // pointer alias.
        let canonical = self.canonical_receiver_record(receiver_type_name);
        self.resolve_named_type(
            &QualIdent {
                span: self.module.span,
                module: None,
                name: canonical,
            },
            scope_type_names,
        )
    }

    /// Resolve the receiver's type as written (no canonicalisation).
    /// Stored on the receiver SYMBOL so inside the method body
    /// `a = b` (where `a: PointerAlias`) type-checks as pointer
    /// comparison rather than "Record = Pointer".  Field-access
    /// `a.field` still works because `apply_selector_type` unwraps
    /// Pointer for Field selectors transparently.
    fn resolve_receiver_symbol_type(
        &self,
        receiver_type_name: &str,
        scope_type_names: &HashSet<String>,
    ) -> SemanticType {
        self.resolve_named_type(
            &QualIdent {
                span: self.module.span,
                module: None,
                name: receiver_type_name.to_string(),
            },
            scope_type_names,
        )
    }

    fn resolve_type_expr(
        &self,
        ty: &TypeExpr,
        scope_type_names: &HashSet<String>,
    ) -> SemanticType {
        self.resolve_type_decl(None, ty, scope_type_names, &[])
    }

    /// Variant that has access to the enclosing procedure's local
    /// symbol table. Needed so array-bound expressions can reference
    /// procedure-scoped CONSTs (e.g. `CONST N = 8; VAR a: ARRAY N OF INTEGER`).
    fn resolve_type_expr_with_locals(
        &self,
        ty: &TypeExpr,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
    ) -> SemanticType {
        self.resolve_type_decl(None, ty, scope_type_names, local_symbols)
    }

    fn resolve_type_decl(
        &self,
        owner_name: Option<&str>,
        ty: &TypeExpr,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
    ) -> SemanticType {
        match ty {
            TypeExpr::QualIdent { ident, .. } => self.resolve_named_type(ident, scope_type_names),
            TypeExpr::Array {
                sys_flag,
                lengths,
                element_type,
                ..
            } => SemanticType::Array {
                lengths: lengths.iter().map(|len_expr| {
                    // Evaluate the length expression to a numeric constant if possible.
                    // Looks up CONSTs declared in the enclosing procedure first
                    // (so `CONST N = 8; VAR a: ARRAY N OF INTEGER` resolves), then
                    // named module-level constants, then IMPORTED constants (so
                    // `ARRAY TextRulers.maxTabs OF INTEGER` folds to its 32-slot
                    // length rather than rendering the qualident name as a string —
                    // which falls through to u64::parse and produces 0).
                    if let Some(ConstValue::Integer(n)) = evaluate_const_expr_with_imports(
                        len_expr,
                        local_symbols,
                        &self.module_symbols,
                        Some(&self.imported_modules),
                    ) {
                        n.to_string()
                    } else {
                        render_expr(len_expr)
                    }
                }).collect(),
                element_type: Box::new(self.resolve_type_decl(None, element_type, scope_type_names, local_symbols)),
                untagged: matches!(self.normalize_sys_flag(sys_flag.as_ref()), Some(NormalizedSysFlag::Untagged)),
            },
            TypeExpr::Record {
                flavor,
                sys_flag,
                base,
                fields,
                ..
            } => SemanticType::Record {
                flavor: *flavor,
                layout: self.record_layout_from_flag(sys_flag.as_ref()),
                base: base
                    .as_ref()
                    .map(|item| Box::new(self.resolve_named_type(item, scope_type_names))),
                fields: fields
                    .iter()
                    .map(|field| self.resolve_field_type(field, scope_type_names))
                    .collect(),
                methods: owner_name
                    .map(|type_name| self.resolve_record_methods(type_name, scope_type_names))
                    .unwrap_or_default(),
            },
            TypeExpr::Pointer { sys_flag, target, .. } => SemanticType::Pointer {
                target: Box::new(self.resolve_type_decl(None, target, scope_type_names, local_symbols)),
                untagged: matches!(self.normalize_sys_flag(sys_flag.as_ref()), Some(NormalizedSysFlag::Untagged)),
            },
            TypeExpr::Procedure {
                formal_parameters,
                ..
            } => SemanticType::Procedure(ProcedureType {
                receiver: None,
                parameters: formal_parameters
                    .as_ref()
                    .map(|parameters| self.resolve_parameter_types(parameters, scope_type_names))
                    .unwrap_or_default(),
                result_type: formal_parameters
                    .as_ref()
                    .and_then(|parameters| parameters.result_type.as_ref())
                    .map(|result| Box::new(self.resolve_type_decl(None, result, scope_type_names, local_symbols))),
                is_new: false,
                flavor: None,
            }),
        }
    }

    fn record_layout_from_flag(&self, flag: Option<&SysFlag>) -> RecordLayout {
        match self.normalize_sys_flag(flag) {
            Some(NormalizedSysFlag::Untagged) => RecordLayout::Untagged,
            Some(NormalizedSysFlag::NoAlign) => RecordLayout::UntaggedNoAlign,
            Some(NormalizedSysFlag::Align2) => RecordLayout::UntaggedAlign2,
            Some(NormalizedSysFlag::Align8) => RecordLayout::UntaggedAlign8,
            Some(NormalizedSysFlag::Union) => RecordLayout::Union,
            _ => RecordLayout::Tagged,
        }
    }

    fn normalize_sys_flag(&self, flag: Option<&SysFlag>) -> Option<NormalizedSysFlag> {
        match flag {
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("untagged") => Some(NormalizedSysFlag::Untagged),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("noalign") => Some(NormalizedSysFlag::NoAlign),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("align2") => Some(NormalizedSysFlag::Align2),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("align8") => Some(NormalizedSysFlag::Align8),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("union") => Some(NormalizedSysFlag::Union),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("nil") => Some(NormalizedSysFlag::Nil),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("ccall") => Some(NormalizedSysFlag::CCall),
            Some(SysFlag::Named(name)) if name.eq_ignore_ascii_case("code") => Some(NormalizedSysFlag::Code),
            _ => None,
        }
    }

    fn resolve_record_methods(
        &self,
        type_name: &str,
        scope_type_names: &HashSet<String>,
    ) -> Vec<MethodType> {
        // CP §11.5: a method receiver may be the record type or its
        // pointer alias.  Match both forms when collecting the
        // record's methods — `(s: SubDesc)` and `(s: Sub)` (where
        // `Sub = POINTER TO SubDesc`) bind to the same record.
        self.module
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Procedure(procedure)
                    if procedure
                        .heading
                        .receiver
                        .as_ref()
                        .is_some_and(|receiver| {
                            self.canonical_receiver_record(&receiver.ty) == type_name
                        }) =>
                {
                    Some(MethodType {
                        name: procedure.heading.name.name.clone(),
                        signature: self.resolve_procedure_signature(scope_type_names, procedure),
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// Canonical record name a receiver type binds to.  When the
    /// receiver is a pointer alias `Foo = POINTER TO FooDesc`, this
    /// returns `"FooDesc"`; for direct record names it returns the
    /// name unchanged.  Cross-module aliases stay unchanged because
    /// methods bind to the record-defining module, not the importer.
    fn canonical_receiver_record(&self, type_name: &str) -> String {
        for declaration in &self.module.declarations {
            let Declaration::Type(type_decl) = declaration else { continue; };
            if type_decl.name.name != type_name { continue; }
            if let TypeExpr::Pointer { target, .. } = &type_decl.ty {
                if let TypeExpr::QualIdent { ident, .. } = target.as_ref() {
                    if ident.module.is_none() {
                        return ident.name.clone();
                    }
                }
            }
            break;
        }
        type_name.to_string()
    }

    fn record_type_info(&self, type_name: &str) -> Option<(Option<RecordFlavor>, Option<String>)> {
        self.module.declarations.iter().find_map(|declaration| match declaration {
            Declaration::Type(type_decl) if type_decl.name.name == type_name => match &type_decl.ty {
                TypeExpr::Record { flavor, base, .. } => Some((*flavor, base.as_ref().map(|item| item.name.clone()))),
                _ => None,
            },
            _ => None,
        })
    }

    /// Follow a chain of `Named` type aliases (local or cross-module)
    /// until we reach the underlying structural type. Used by binary-
    /// operand validation so a CP `INTEGER` literal compares cleanly
    /// against a parameter typed by an imported alias like
    /// `Stores.Store = INTEGER`. Returns the original type when no
    /// alias chain applies (so non-Named types pass through untouched).
    fn unwrap_named_aliases(
        &self,
        ty: SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> SemanticType {
        let mut current = ty;
        let mut seen: HashSet<String> = HashSet::new();
        loop {
            let resolved = resolve_named_type_alias(
                &current,
                local_symbols,
                &self.module_symbols,
                &self.imported_modules,
                &mut seen,
            )
            .cloned();
            match resolved {
                Some(next) if next != current => current = next,
                _ => return current,
            }
        }
    }

    fn resolve_alias_to_builtin_target(&self, name: &str) -> Option<BuiltinType> {
        let mut current = name.to_string();
        for _ in 0..16 {
            let decl = self
                .module
                .declarations
                .iter()
                .find_map(|declaration| match declaration {
                    Declaration::Type(type_decl) if type_decl.name.name == current => {
                        Some(type_decl)
                    }
                    _ => None,
                })?;
            match &decl.ty {
                TypeExpr::QualIdent { ident, .. } if ident.module.is_none() => {
                    if let Some(builtin) = builtin_type_by_name(&ident.name) {
                        return Some(builtin);
                    }
                    current = ident.name.clone();
                }
                _ => return None,
            }
        }
        None
    }

    fn find_record_decl(&self, type_name: &str) -> Option<&'a TypeDecl> {
        let direct = self.module.declarations.iter().find_map(|declaration| match declaration {
            Declaration::Type(type_decl) if type_decl.name.name == type_name => Some(type_decl),
            _ => None,
        })?;
        // CP §11.5: a method receiver may be either the record type or
        // its pointer alias.  When the lookup landed on a pointer
        // alias (`Foo = POINTER TO FooDesc`), chase one level to the
        // record itself so the caller's `record_flavor` / `base_name`
        // checks see the underlying record's shape.  Without this every
        // BlackBox-style method declaration `(s: Foo) Method` gets
        // mis-flagged as "record Foo must be ABSTRACT" because the
        // pointer alias matches no flavor.
        if let TypeExpr::Pointer { target, .. } = &direct.ty {
            if let TypeExpr::QualIdent { ident, .. } = target.as_ref() {
                if ident.module.is_none() {
                    if let Some(target_decl) = self.module.declarations.iter().find_map(|declaration| match declaration {
                        Declaration::Type(type_decl) if type_decl.name.name == ident.name => Some(type_decl),
                        _ => None,
                    }) {
                        return Some(target_decl);
                    }
                }
            }
        }
        Some(direct)
    }

    fn find_inherited_method(&self, type_name: &str, method_name: &str) -> Option<&'a ProcedureDecl> {
        // Normalize the starting receiver name to its underlying record
        // type so callers passing the pointer-alias form (e.g. `Sub`)
        // walk the same chain as the Desc form (`SubDesc`).
        let canonical_start = self.canonical_receiver_record(type_name);

        // Walk the base chain using the QUALIFIED base info. When the
        // base is in another module (`module: Some("Views")`), stop
        // here — cross-module inheritance is the caller's
        // `has_inherited_method_anywhere` path, not ours. Without this
        // stop, `record_type_info` strips the module qualifier and
        // every `base.name` like `Views.ViewDesc` collapses to plain
        // `ViewDesc`; if the LOCAL module happens to declare a record
        // ALSO named `ViewDesc` (which is exactly what happens when a
        // module like `Containers` extends `Views.ViewDesc` with its
        // own `ContainersViewDesc` — sema would walk into the local
        // record and find its OWN methods as "inherited from itself").
        let mut current = self
            .record_type_info_qualified(&canonical_start)
            .and_then(|(_, base)| base);
        while let Some((base_module, base_name)) = current {
            if base_module.is_some() {
                // Cross-module base. Stop the local walk — the caller
                // handles imported-method discovery via
                // `has_inherited_method_anywhere`.
                return None;
            }

            // Match procedures whose receiver — written as either the
            // record-Desc form OR a pointer alias to it — binds to
            // `base_name`.
            if let Some(method) = self.module.declarations.iter().find_map(|declaration| match declaration {
                Declaration::Procedure(procedure)
                    if procedure.heading.receiver.as_ref().is_some_and(|receiver| {
                        self.canonical_receiver_record(&receiver.ty) == base_name
                    })
                        && procedure.heading.name.name == method_name =>
                {
                    Some(procedure)
                }
                _ => None,
            }) {
                return Some(method);
            }
            current = self
                .record_type_info_qualified(&base_name)
                .and_then(|(_, base)| base);
        }
        None
    }

    /// Walks the inheritance chain (including bases declared in imported
    /// modules) and returns true if any ancestor record has a method
    /// named `method_name`. Used by the override-detection check so a
    /// subclass method that overrides an inherited cross-module method
    /// is not flagged with "newly introduced method must use NEW".
    fn has_inherited_method_anywhere(&self, type_name: &str, method_name: &str) -> bool {
        // Walk local bases first (matches find_inherited_method behaviour).
        let mut current_qualified = self.record_type_info_qualified(type_name).and_then(|(_, base)| base);
        while let Some((module_qual, base_name)) = current_qualified.clone() {
            if module_qual.is_none() {
                // Local base. Look for a same-module ProcedureDecl.
                let local_hit = self.module.declarations.iter().any(|declaration| match declaration {
                    Declaration::Procedure(procedure) => procedure
                        .heading
                        .receiver
                        .as_ref()
                        .is_some_and(|receiver| receiver.ty == base_name)
                        && procedure.heading.name.name == method_name,
                    _ => false,
                });
                if local_hit {
                    return true;
                }
                current_qualified = self
                    .record_type_info_qualified(&base_name)
                    .and_then(|(_, base)| base);
            } else if let Some(module_name) = module_qual {
                // Cross-module base: walk the imported record's methods,
                // then chase its base recursively. The imported record
                // type is reachable as a SemanticType in
                // self.imported_modules; its methods carry the names we
                // need for the override check.
                return self.imported_record_inherits_method(
                    &module_name,
                    &base_name,
                    method_name,
                );
            }
        }
        false
    }

    /// Recursively checks whether the record `record_name` declared in
    /// `module_name` (or any of its bases) has a method named
    /// `method_name`. The base may itself be in a third module — handled
    /// by following the qualified Named refs left in place by
    /// `qualify_local_named_refs`.
    fn imported_record_inherits_method(
        &self,
        module_name: &str,
        record_name: &str,
        method_name: &str,
    ) -> bool {
        let Some(symbols) = self.imported_modules.get(module_name) else {
            return false;
        };
        let Some(record_sym) = symbols
            .iter()
            .find(|sym| sym.kind == SymbolKind::Type && sym.name == record_name)
        else {
            return false;
        };
        let Some(record_ty) = record_sym.declared_type.as_ref() else {
            return false;
        };
        let Some(SemanticType::Record { methods, base, .. }) = unwrap_to_record(record_ty) else {
            return false;
        };
        if methods.iter().any(|m| m.name == method_name) {
            return true;
        }
        // Walk into the base.
        match base.as_deref() {
            Some(SemanticType::Named { module: Some(base_module), name: base_name, .. }) => {
                self.imported_record_inherits_method(base_module, base_name, method_name)
            }
            Some(SemanticType::Named { module: None, name: base_name, .. }) => {
                // Local-to-the-imported-module base. Same module.
                self.imported_record_inherits_method(module_name, base_name, method_name)
            }
            _ => false,
        }
    }

    /// Like `record_type_info` but returns the base's full qualification
    /// (`(module, name)`) instead of just the unqualified name. Required
    /// for the cross-module inheritance walks.
    fn record_type_info_qualified(
        &self,
        type_name: &str,
    ) -> Option<(Option<RecordFlavor>, Option<(Option<String>, String)>)> {
        self.module.declarations.iter().find_map(|declaration| match declaration {
            Declaration::Type(type_decl) if type_decl.name.name == type_name => match &type_decl.ty {
                TypeExpr::Record { flavor, base, .. } => Some((
                    *flavor,
                    base.as_ref().map(|item| (item.module.clone(), item.name.clone())),
                )),
                _ => None,
            },
            _ => None,
        })
    }

    fn effective_methods_for_type(&self, type_name: &str) -> Vec<&'a ProcedureDecl> {
        // Walk the LOCAL inheritance chain only.  `record_type_info`
        // strips the module qualifier off the base, so when a local
        // record happens to share its name with an imported one
        // (e.g. `TextModels.UpdateMsg` extends `Models.UpdateMsg`),
        // recursing with the stripped name circles back onto the
        // type currently being analysed and spins forever.  Use the
        // qualified form: if the base is cross-module, stop here
        // (the local pass picks up only local-receiver methods,
        // which is all this caller wants).
        let mut methods = match self.record_type_info_qualified(type_name) {
            Some((_, Some((Some(_module), _name)))) => Vec::new(),
            Some((_, Some((None, base)))) => self.effective_methods_for_type(&base),
            _ => Vec::new(),
        };

        for declaration in &self.module.declarations {
            let Declaration::Procedure(procedure) = declaration else {
                continue;
            };
            if !procedure
                .heading
                .receiver
                .as_ref()
                .is_some_and(|receiver| receiver.ty == type_name)
            {
                continue;
            }

            if let Some(existing_index) = methods
                .iter()
                .position(|item| item.heading.name.name == procedure.heading.name.name)
            {
                methods[existing_index] = procedure;
            } else {
                methods.push(procedure);
            }
        }

        methods
    }

    fn resolve_field_type(
        &self,
        field: &FieldDecl,
        scope_type_names: &HashSet<String>,
    ) -> FieldType {
        FieldType {
            names: field.names.iter().map(|item| item.name.clone()).collect(),
            ty: self.resolve_type_expr(&field.ty, scope_type_names),
        }
    }

    fn resolve_named_type(
        &self,
        ident: &QualIdent,
        _scope_type_names: &HashSet<String>,
    ) -> SemanticType {
        if ident.module.is_none() {
            // CP §6.3 scoping: a user-defined TYPE in scope shadows any
            // builtin with the same name.  This matters for the pseudo-
            // builtin pseudo-types "String" / "Shortstring" (internal
            // labels for multi-char string-literal types) — when CP code
            // does `TYPE String = ARRAY N OF CHAR`, that alias must win,
            // otherwise downstream code that indexes `: String` field
            // sees the opaque builtin instead of an array.  Surfaced by
            // TextMappers.Scanner's `string*: String` field and
            // Scanner.Scan's `s.string[i] := ch` write.
            //
            // Note: scope_type_names already includes every builtin name
            // (CHAR / INTEGER / BOOLEAN / …), so we can't gate on that
            // set alone — that would re-route primitives through the
            // user-alias path and break the type system.  Check
            // module-level TYPE declarations directly instead.
            let user_declared = self
                .module
                .declarations
                .iter()
                .any(|d| matches!(d, Declaration::Type(t) if t.name.name == ident.name));
            if user_declared {
                if let Some(builtin) = self.resolve_alias_to_builtin_target(&ident.name) {
                    return SemanticType::Builtin(builtin);
                }
                return SemanticType::Named {
                    module: None,
                    name: ident.name.clone(),
                    kind: NamedTypeKind::UserDefined,
                };
            }
            if let Some(builtin) = builtin_type_by_name(&ident.name) {
                return SemanticType::Builtin(builtin);
            }
        }

        SemanticType::Named {
            module: ident.module.clone(),
            name: ident.name.clone(),
            kind: if ident.module.is_some() {
                NamedTypeKind::Imported
            } else {
                NamedTypeKind::Unresolved
            },
        }
    }

    fn walk_statements(
        &self,
        statements: &[Statement],
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
        expected_result_type: Option<&SemanticType>,
        resolutions: &mut Vec<SelectorResolution>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        for statement in statements {
            match statement {
                Statement::Empty { .. } | Statement::Exit { .. } | Statement::Brk { .. } => {
                    // BRK / BRK(expr): no sema constraints.  The IR
                    // layer lowers the optional pointer expression
                    // through its own lower_expr path which performs
                    // designator resolution.
                }
                Statement::Assignment { target, value, .. } => {
                    self.walk_designator_expressions(
                        target,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    self.record_designator_resolution(target, procedure_name, scope_type_names, resolutions);
                    self.validate_designator(target, procedure_name, local_symbols, scope_type_names, diagnostics);
                    self.validate_assignment_target(target, procedure_name, local_symbols, diagnostics);
                    self.walk_expr(
                        value,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    if let (Some(target_type), Some(value_type)) = (
                        self.infer_designator_type(target, local_symbols, scope_type_names),
                        self.infer_expr_type(value, local_symbols, scope_type_names),
                    ) {
                        let compatible = self.types_are_assignment_compatible(
                            &target_type,
                            &value_type,
                            local_symbols,
                        ) || integer_literal_fits_target(value, &target_type);
                        if !compatible {
                            let (line, column) = statement_position(statement);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                format!(
                                    "assignment type mismatch: expected {}, found {}",
                                    render_semantic_type(&target_type),
                                    render_semantic_type(&value_type)
                                ),
                            ));
                        }
                    }
                }
                Statement::ProcedureCall { designator, .. } => {
                    self.walk_designator_expressions(
                        designator,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    self.record_designator_resolution(designator, procedure_name, scope_type_names, resolutions);
                    self.validate_procedure_call_statement(
                        designator,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                }
                Statement::If {
                    branches,
                    else_branch,
                    ..
                } => {
                    for branch in branches {
                        self.walk_expr(
                            &branch.condition,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            resolutions,
                            diagnostics,
                        );
                        self.require_boolean_expr(
                            &branch.condition,
                            procedure_name,
                            local_symbols,
                            scope_type_names,
                            diagnostics,
                            "IF condition must be BOOLEAN",
                        );
                        self.walk_statements(
                            &branch.body,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            expected_result_type,
                            resolutions,
                            diagnostics,
                        );
                    }
                    if let Some(else_branch) = else_branch {
                        self.walk_statements(
                            else_branch,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            expected_result_type,
                            resolutions,
                            diagnostics,
                        );
                    }
                }
                Statement::Case {
                    expr,
                    arms,
                    else_branch,
                    ..
                } => {
                    self.walk_expr(
                        expr,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    for arm in arms {
                        for label in &arm.labels {
                            self.walk_expr(
                                &label.start,
                                procedure_name,
                                scope_type_names,
                                local_symbols,
                                resolutions,
                                diagnostics,
                            );
                            if let Some(end) = &label.end {
                                self.walk_expr(
                                    end,
                                    procedure_name,
                                    scope_type_names,
                                    local_symbols,
                                    resolutions,
                                    diagnostics,
                                );
                            }
                        }
                        self.walk_statements(
                            &arm.body,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            expected_result_type,
                            resolutions,
                            diagnostics,
                        );
                    }
                    self.validate_case_statement(
                        expr,
                        arms,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        diagnostics,
                    );
                    if let Some(else_branch) = else_branch {
                        self.walk_statements(
                            else_branch,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            expected_result_type,
                            resolutions,
                            diagnostics,
                        );
                    }
                }
                Statement::While { condition, body, .. } => {
                    self.walk_expr(
                        condition,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    self.require_boolean_expr(
                        condition,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                        "WHILE condition must be BOOLEAN",
                    );
                    self.walk_statements(
                        body,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        expected_result_type,
                        resolutions,
                        diagnostics,
                    );
                }
                Statement::Repeat { body, until, .. } => {
                    self.walk_statements(
                        body,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        expected_result_type,
                        resolutions,
                        diagnostics,
                    );
                    self.walk_expr(
                        until,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    self.require_boolean_expr(
                        until,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                        "REPEAT condition must be BOOLEAN",
                    );
                }
                Statement::For {
                    variable,
                    start,
                    end,
                    step,
                    body,
                    ..
                } => {
                    self.walk_expr(
                        start,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    self.walk_expr(
                        end,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    if let Some(step) = step {
                        self.walk_expr(
                            step,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            resolutions,
                            diagnostics,
                        );
                    }
                    self.require_integer_expr(
                        start,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                        "FOR initial value must be an integer",
                    );
                    self.require_integer_expr(
                        end,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                        "FOR final value must be an integer",
                    );
                    if let Some(step) = step {
                        self.require_integer_expr(
                            step,
                            procedure_name,
                            local_symbols,
                            scope_type_names,
                            diagnostics,
                            "FOR step must be an integer",
                        );
                        match evaluate_const_integer(step, local_symbols, &self.module_symbols) {
                            Some(ConstValue::Integer(0)) => {
                                let (line, column) = expr_position(step);
                                diagnostics.push(make_diagnostic(
                                    procedure_name,
                                    line,
                                    column,
                                    "FOR step must be nonzero".to_string(),
                                ));
                            }
                            None => {
                                let (line, column) = expr_position(step);
                                diagnostics.push(make_diagnostic(
                                    procedure_name,
                                    line,
                                    column,
                                    "FOR step must be a constant expression".to_string(),
                                ));
                            }
                            Some(_) => {}
                        }
                    }
                    if let Some(variable_type) = self.lookup_symbol_type(variable, local_symbols) {
                        if !is_integer_type(&variable_type) {
                            let (line, column) = statement_position(statement);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                format!(
                                    "FOR control variable {} must be an integer, found {}",
                                    variable,
                                    render_semantic_type(&variable_type)
                                ),
                            ));
                        }
                    } else {
                        let (line, column) = statement_position(statement);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!("FOR control variable {} is not declared", variable),
                        ));
                    }
                    self.walk_statements(
                        body,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        expected_result_type,
                        resolutions,
                        diagnostics,
                    );
                }
                Statement::Loop { body, .. } => {
                    self.walk_statements(
                        body,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        expected_result_type,
                        resolutions,
                        diagnostics,
                    );
                }
                Statement::With {
                    arms,
                    else_branch,
                    ..
                } => {
                    for arm in arms {
                        // For a guarded arm `WITH v: T DO body`, narrow
                        // `v` to type T inside the body so field /
                        // method lookups resolve against T's layout.
                        // We push a shadowing symbol with the narrowed
                        // type and pop it after walking.  CP §9.3:
                        // within the body, `v` is treated as having the
                        // guard type, but only for the body's lifetime.
                        let mut narrowed_locals: Vec<SemanticSymbol> = local_symbols.to_vec();
                        if let Some(guard) = &arm.guard {
                            self.walk_guard(guard, procedure_name, scope_type_names, resolutions);
                            self.validate_with_guard(
                                guard,
                                procedure_name,
                                scope_type_names,
                                local_symbols,
                                diagnostics,
                            );
                            // Look up the original symbol to inherit
                            // mode/exported flags; only the type changes.
                            if let Some(orig) = local_symbols
                                .iter()
                                .rev()
                                .find(|s| s.name == guard.variable.name)
                            {
                                let narrowed_ty = self.resolve_type_decl(
                                    None,
                                    &TypeExpr::QualIdent {
                                        span: guard.ty.span,
                                        ident: guard.ty.clone(),
                                    },
                                    scope_type_names,
                                    local_symbols,
                                );
                                // CP §9.3: a record-typed guard variable
                                // stays a record; a pointer-typed one
                                // stays a pointer (to the narrowed
                                // record).  Wrap the resolved record in
                                // a Pointer if the original symbol was
                                // pointer-typed.
                                let final_ty = match orig.declared_type.as_ref() {
                                    Some(SemanticType::Pointer { untagged, .. }) => SemanticType::Pointer {
                                        target: Box::new(narrowed_ty),
                                        untagged: *untagged,
                                    },
                                    _ => narrowed_ty,
                                };
                                let mut narrowed = orig.clone();
                                narrowed.declared_type = Some(final_ty);
                                narrowed_locals.push(narrowed);
                            }
                        }
                        self.walk_statements(
                            &arm.body,
                            procedure_name,
                            scope_type_names,
                            &narrowed_locals,
                            expected_result_type,
                            resolutions,
                            diagnostics,
                        );
                    }
                    if let Some(else_branch) = else_branch {
                        self.walk_statements(
                            else_branch,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            expected_result_type,
                            resolutions,
                            diagnostics,
                        );
                    }
                }
                Statement::Return { expr, .. } => {
                    if let Some(expr) = expr {
                        self.walk_expr(
                            expr,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            resolutions,
                            diagnostics,
                        );
                    }
                    match (expected_result_type, expr.as_ref()) {
                        (None, Some(_)) => {
                            let (line, column) = statement_position(statement);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                "procedure without result type cannot return a value".to_string(),
                            ))
                        }
                        (Some(expected), Some(expr)) => {
                            if let Some(actual) =
                                self.infer_expr_type(expr, local_symbols, scope_type_names)
                            {
                                // CP §11.5 receiver-return special case:
                                // `(b: Box) M (): Box; RETURN b` is valid
                                // even though sema canonicalises the
                                // receiver to its underlying record for
                                // dispatch matching, which makes `b` look
                                // like Record(BoxDesc) here and the
                                // declared return type Pointer(Named).
                                // Accept the pair when the expression IS
                                // a receiver designator AND its record
                                // matches the expected pointer's target.
                                let receiver_round_trip = self
                                    .return_is_receiver_pointer_round_trip(
                                        expr,
                                        expected,
                                        &actual,
                                        local_symbols,
                                    );
                                if !receiver_round_trip
                                    && !self.types_are_assignment_compatible(
                                        expected,
                                        &actual,
                                        local_symbols,
                                    )
                                {
                                    let (line, column) = statement_position(statement);
                                    diagnostics.push(make_diagnostic(
                                        procedure_name,
                                        line,
                                        column,
                                        format!(
                                            "return type mismatch: expected {}, found {}",
                                            render_semantic_type(expected),
                                            render_semantic_type(&actual)
                                        ),
                                    ));
                                }
                            }
                        }
                        (Some(expected), None) => {
                            let (line, column) = statement_position(statement);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                format!(
                                    "function must return a value of type {}",
                                    render_semantic_type(expected)
                                ),
                            ))
                        }
                        (None, None) => {}
                    }
                }
            }
        }
    }

    fn validate_case_statement(
        &self,
        expr: &Expr,
        arms: &[newcp_parser::CaseArm],
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let Some(selector_type) = self.infer_expr_type(expr, local_symbols, scope_type_names) else {
            return;
        };

        if !is_case_selector_type(&selector_type) {
            let (line, column) = expr_position(expr);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!(
                    "CASE expression must be an integer or character type; found {}",
                    render_semantic_type(&selector_type)
                ),
            ));
            return;
        }

        let mut seen_ranges = Vec::new();
        for arm in arms {
            for label in &arm.labels {
                if !self.is_case_label_constant(&label.start, local_symbols) {
                    let (line, column) = expr_position(&label.start);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        "CASE label must be a constant expression".to_string(),
                    ));
                }

                let Some(start_type) = self.infer_expr_type(&label.start, local_symbols, scope_type_names) else {
                    continue;
                };

                if !self.types_are_assignment_compatible(&selector_type, &start_type, local_symbols) {
                    let (line, column) = expr_position(&label.start);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!(
                            "CASE label type {} is not compatible with selector type {}",
                            render_semantic_type(&start_type),
                            render_semantic_type(&selector_type)
                        ),
                    ));
                }

                let start_value = evaluate_case_label_value(&label.start, local_symbols);
                let end_value = match &label.end {
                    Some(end) => {
                        if !self.is_case_label_constant(end, local_symbols) {
                            let (line, column) = expr_position(end);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                "CASE label must be a constant expression".to_string(),
                            ));
                        }

                        if let Some(end_type) = self.infer_expr_type(end, local_symbols, scope_type_names) {
                            if !self.types_are_assignment_compatible(
                                &selector_type,
                                &end_type,
                                local_symbols,
                            ) {
                                let (line, column) = expr_position(end);
                                diagnostics.push(make_diagnostic(
                                    procedure_name,
                                    line,
                                    column,
                                    format!(
                                        "CASE label type {} is not compatible with selector type {}",
                                        render_semantic_type(&end_type),
                                        render_semantic_type(&selector_type)
                                    ),
                                ));
                            }
                        }

                        evaluate_case_label_value(end, local_symbols)
                    }
                    None => None,
                };

                let Some(start_value) = start_value else {
                    continue;
                };
                let end_value = end_value.unwrap_or(start_value);

                if start_value.kind != end_value.kind {
                    let (line, column) = expr_position(&label.start);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        "CASE label range endpoints must have matching types".to_string(),
                    ));
                    continue;
                }

                if start_value.value > end_value.value {
                    let (line, column) = expr_position(&label.start);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        "CASE label range start must not be greater than its end".to_string(),
                    ));
                    continue;
                }

                if let Some(previous) = seen_ranges.iter().find(|previous: &&EvaluatedCaseRange| {
                    previous.kind == start_value.kind
                        && previous.start <= end_value.value
                        && start_value.value <= previous.end
                }) {
                    let (line, column) = expr_position(&label.start);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!(
                            "CASE label {} overlaps earlier label {}",
                            render_case_value_range(start_value.value, end_value.value, start_value.kind),
                            render_case_value_range(previous.start, previous.end, previous.kind)
                        ),
                    ));
                    continue;
                }

                seen_ranges.push(EvaluatedCaseRange {
                    kind: start_value.kind,
                    start: start_value.value,
                    end: end_value.value,
                });
            }
        }
    }

    fn is_case_label_constant(&self, expr: &Expr, local_symbols: &[SemanticSymbol]) -> bool {
        match expr {
            // Single-character string literals (e.g. "a") are CHAR constants — valid CASE labels.
            // Multi-character strings and reals are not.
            Expr::Literal { value, .. } => match value {
                newcp_parser::Literal::String(s) => s.trim_matches(['\'', '"']).chars().count() == 1,
                newcp_parser::Literal::Real(_) => false,
                _ => true,
            },
            Expr::Designator(designator) => {
                designator.base.module.is_none()
                    && designator.selectors.is_empty()
                    && self
                        .lookup_symbol(&designator.base.name, local_symbols)
                        .is_some_and(|symbol| symbol.kind == SymbolKind::Constant)
            }
            Expr::Unary { op, expr, .. } => {
                matches!(op, newcp_parser::UnaryOp::Plus | newcp_parser::UnaryOp::Minus)
                    && self.is_case_label_constant(expr, local_symbols)
            }
            Expr::Binary { left, op, right, .. } => {
                matches!(
                    op,
                    BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Div
                        | BinaryOp::Mod
                ) && self.is_case_label_constant(left, local_symbols)
                    && self.is_case_label_constant(right, local_symbols)
            }
            _ => false,
        }
    }

    fn walk_guard(
        &self,
        guard: &Guard,
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        resolutions: &mut Vec<SelectorResolution>,
    ) {
        let resolved_type = self.resolve_named_type(&guard.ty, scope_type_names);
        resolutions.push(SelectorResolution {
            procedure: procedure_name.map(str::to_string),
            designator: format!("guard {}:{}", render_qualident(&guard.variable), render_qualident(&guard.ty)),
            selector: render_qualident(&guard.ty),
            kind: selector_resolution_kind_for_type(&resolved_type),
            reason: format!("guard target resolves as {}", render_semantic_type(&resolved_type)),
        });
    }

    fn validate_with_guard(
        &self,
        guard: &Guard,
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        if guard.variable.module.is_some() {
            let (line, column) = (guard.span.start.line, guard.span.start.column);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                "WITH guard variable must be a local designator".to_string(),
            ));
            return;
        }

        let Some(subject_symbol) = self.lookup_symbol(&guard.variable.name, local_symbols) else {
            let (line, column) = (guard.span.start.line, guard.span.start.column);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("WITH guard variable {} is not declared", guard.variable.name),
            ));
            return;
        };

        let Some(subject_type) = subject_symbol.declared_type.as_ref() else {
            return;
        };

        self.validate_type_test_operands(
            TypeTestContext::WithGuard(&guard.variable.name),
            Some(subject_symbol),
            subject_type,
            &guard.ty,
            procedure_name,
            scope_type_names,
            local_symbols,
            (guard.span.start.line, guard.span.start.column),
            diagnostics,
        );
    }

    fn walk_expr(
        &self,
        expr: &Expr,
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
        resolutions: &mut Vec<SelectorResolution>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        match expr {
            Expr::Literal { .. } | Expr::Nil { .. } => {}
            Expr::Designator(designator) => {
                self.walk_designator_expressions(
                    designator,
                    procedure_name,
                    scope_type_names,
                    local_symbols,
                    resolutions,
                    diagnostics,
                );
                self.record_designator_resolution(designator, procedure_name, scope_type_names, resolutions);
                self.validate_designator(
                    designator,
                    procedure_name,
                    local_symbols,
                    scope_type_names,
                    diagnostics,
                );
            }
            Expr::Set { elements, .. } => {
                for element in elements {
                    self.walk_expr(
                        &element.start,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        resolutions,
                        diagnostics,
                    );
                    if let Some(end) = &element.end {
                        self.walk_expr(
                            end,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            resolutions,
                            diagnostics,
                        );
                    }
                }
            }
            Expr::Unary { expr, .. } => self.walk_expr(
                expr,
                procedure_name,
                scope_type_names,
                local_symbols,
                resolutions,
                diagnostics,
            ),
            Expr::Binary { left, right, op, .. } => {
                self.walk_expr(
                    left,
                    procedure_name,
                    scope_type_names,
                    local_symbols,
                    resolutions,
                    diagnostics,
                );
                self.walk_expr(
                    right,
                    procedure_name,
                    scope_type_names,
                    local_symbols,
                    resolutions,
                    diagnostics,
                );
                if *op == BinaryOp::Is {
                    if let Expr::Designator(designator) = right.as_ref() {
                        self.record_designator_resolution(designator, procedure_name, scope_type_names, resolutions);
                    }
                }
            }
        }

        self.validate_expr_operators(
            expr,
            procedure_name,
            local_symbols,
            scope_type_names,
            diagnostics,
        );
    }

    fn require_boolean_expr(
        &self,
        expr: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
        message: &str,
    ) {
        if let Some(expr_type) = self.infer_expr_type(expr, local_symbols, scope_type_names) {
            if !is_boolean_type(&expr_type) {
                let (line, column) = expr_position(expr);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!("{}; found {}", message, render_semantic_type(&expr_type)),
                ));
            }
        }
    }

    fn require_integer_expr(
        &self,
        expr: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
        message: &str,
    ) {
        if let Some(expr_type) = self.infer_expr_type(expr, local_symbols, scope_type_names) {
            if !is_integer_type(&expr_type) {
                let (line, column) = expr_position(expr);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!("{}; found {}", message, render_semantic_type(&expr_type)),
                ));
            }
        }
    }

    fn infer_expr_type(
        &self,
        expr: &Expr,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
    ) -> Option<SemanticType> {
        match expr {
            Expr::Literal { value, .. } => match value {
                newcp_parser::Literal::Integer(value) => Some(integer_literal_type(value)),
                newcp_parser::Literal::Real(_) => Some(SemanticType::Builtin(BuiltinType::Real)),
                newcp_parser::Literal::Character(value) => Some(character_literal_type(value)),
                newcp_parser::Literal::String(value) => Some(string_literal_type(value)),
            },
            Expr::Nil { .. } => Some(SemanticType::Nil),
            Expr::Designator(designator) => {
                self.infer_designator_type(designator, local_symbols, scope_type_names)
            }
            Expr::Set { .. } => Some(SemanticType::Builtin(BuiltinType::Set)),
            Expr::Unary { op, expr, .. } => {
                let inner = self.infer_expr_type(expr, local_symbols, scope_type_names)?;
                match op {
                    newcp_parser::UnaryOp::Plus | newcp_parser::UnaryOp::Minus => {
                        if is_numeric_type(&inner) {
                            Some(inner)
                        } else {
                            None
                        }
                    }
                    newcp_parser::UnaryOp::Not => {
                        if is_boolean_type(&inner) {
                            Some(SemanticType::Builtin(BuiltinType::Boolean))
                        } else {
                            None
                        }
                    }
                }
            }
            Expr::Binary { left, op, right, .. } => {
                let left_type = self.infer_expr_type(left, local_symbols, scope_type_names);
                let right_type = self.infer_expr_type(right, local_symbols, scope_type_names);
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {
                        infer_additive_or_multiplicative_result(*op, left_type.as_ref(), right_type.as_ref())
                    }
                    BinaryOp::Divide => {
                        // SET / SET = symmetric difference (→ SET)
                        if matches!(left_type.as_ref(), Some(SemanticType::Builtin(BuiltinType::Set)))
                            && matches!(right_type.as_ref(), Some(SemanticType::Builtin(BuiltinType::Set)))
                        {
                            Some(SemanticType::Builtin(BuiltinType::Set))
                        } else if left_type.as_ref().is_some_and(|ty| is_numeric_type(ty))
                            && right_type.as_ref().is_some_and(|ty| is_numeric_type(ty))
                        {
                            Some(SemanticType::Builtin(BuiltinType::Real))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Div | BinaryOp::Mod => {
                        if left_type.as_ref().is_some_and(|ty| is_integer_type(ty))
                            && right_type.as_ref().is_some_and(|ty| is_integer_type(ty))
                        {
                            Some(promote_integer_pair(left_type.as_ref()?, right_type.as_ref()?))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Or | BinaryOp::And => {
                        if left_type.as_ref().is_some_and(|ty| is_boolean_type(ty))
                            && right_type.as_ref().is_some_and(|ty| is_boolean_type(ty))
                        {
                            Some(SemanticType::Builtin(BuiltinType::Boolean))
                        } else {
                            None
                        }
                    }
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::In
                    | BinaryOp::Is => Some(SemanticType::Builtin(BuiltinType::Boolean)),
                }
            }
        }
    }

    fn infer_designator_type(
        &self,
        designator: &Designator,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
    ) -> Option<SemanticType> {
        let designator = self.normalize_designator(designator, local_symbols);
        let mut current = if designator.base.module.is_none() {
            match designator.base.name.as_str() {
                "TRUE" | "FALSE" => Some(SemanticType::Builtin(BuiltinType::Boolean)),
                "INF" => Some(SemanticType::Builtin(BuiltinType::Real)),
                _ if builtin_proc_by_name(&designator.base.name).is_some() => {
                    Some(SemanticType::BuiltinProc(
                        builtin_proc_by_name(&designator.base.name).expect("builtin checked"),
                    ))
                }
                _ => self.lookup_symbol_type(&designator.base.name, local_symbols),
            }
        } else {
            self.resolve_system_builtin(designator.base.module.as_deref(), &designator.base.name)
                .map(SemanticType::BuiltinProc)
        }?;

        for selector in &designator.selectors {
            current = self.apply_selector_type(&current, selector, scope_type_names)?;
        }

        Some(current)
    }

    fn validate_assignment_target(
        &self,
        designator: &Designator,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let designator = self.normalize_designator(designator, local_symbols);
        let (line, column) = designator_position(&designator);

        // CP §8.4: a designator is an l-value when its FINAL selector
        // yields a variable (Field / Index / Dereference / TypeGuard /
        // AmbiguousParen-as-TypeGuard). Intermediate TypeGuard /
        // AmbiguousParen selectors only narrow the static type — the
        // underlying storage is still the same variable, so the chain
        // can still be assigned through. Without this exemption,
        // `b(Sub).field := value` (BlackBox-style narrowing pattern)
        // is rejected, forcing callers to introduce an intermediate
        // typed temporary.
        //
        // A trailing `Selector::Call(_)` IS rejected — the result of a
        // call has no general storage backing.
        if matches!(designator.selectors.last(), Some(Selector::Call(_))) {
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                "assignment target is not assignable".to_string(),
            ));
            return;
        }

        if designator.base.module.is_some() {
            return;
        }

        if let Some(symbol) = self.lookup_symbol(&designator.base.name, local_symbols) {
            if !matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Parameter | SymbolKind::Receiver) {
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "assignment target must be a variable, parameter, or receiver, found {} {}",
                        render_symbol_kind(symbol.kind),
                        symbol.name
                    ),
                ));
            }
            // CP §10.1.1: a parameter declared `IN` is read-only.
            // Reject any assignment whose root is an IN parameter,
            // including writes through fields (`p.field`) and indexed
            // elements (`p[i]`). This conservative rule matches the
            // BlackBox compiler's enforcement.
            if symbol.param_mode == Some(ParamMode::In) {
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "cannot assign through IN parameter '{}' — \
                         IN parameters are read-only (use VAR if the \
                         callee needs to mutate the caller's data, or \
                         OUT if the parameter is purely an output)",
                        symbol.name
                    ),
                ));
            }
        }
    }

    fn validate_expr_operators(
        &self,
        expr: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        match expr {
            Expr::Unary { op, expr, .. } => {
                if let Some(inner) = self.infer_expr_type(expr, local_symbols, scope_type_names) {
                    let valid = match op {
                        newcp_parser::UnaryOp::Plus => is_numeric_type(&inner),
                        newcp_parser::UnaryOp::Minus => {
                            is_numeric_type(&inner) || matches!(inner, SemanticType::Builtin(BuiltinType::Set))
                        }
                        newcp_parser::UnaryOp::Not => is_boolean_type(&inner),
                    };
                    if !valid {
                        let (line, column) = expr_position(expr.as_ref());
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!(
                                "invalid unary operator {} for {}",
                                render_unary_op(*op),
                                render_semantic_type(&inner)
                            ),
                        ));
                    }
                }
            }
            Expr::Binary { left, op, right, .. } => {
                if *op == BinaryOp::Is {
                    self.validate_is_expression(
                        left,
                        right,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                    return;
                }

                let left_type = self
                    .infer_expr_type(left, local_symbols, scope_type_names)
                    .map(|t| self.unwrap_named_aliases(t, local_symbols));
                let right_type = self
                    .infer_expr_type(right, local_symbols, scope_type_names)
                    .map(|t| self.unwrap_named_aliases(t, local_symbols));
                if let (Some(left_type), Some(right_type)) = (left_type, right_type) {
                    let valid = match op {
                        BinaryOp::Add => {
                            (is_numeric_type(&left_type) && is_numeric_type(&right_type))
                                || matches!((&left_type, &right_type), (SemanticType::Builtin(BuiltinType::Set), SemanticType::Builtin(BuiltinType::Set)))
                        }
                        BinaryOp::Subtract | BinaryOp::Multiply => {
                            (is_numeric_type(&left_type) && is_numeric_type(&right_type))
                                || matches!((&left_type, &right_type), (SemanticType::Builtin(BuiltinType::Set), SemanticType::Builtin(BuiltinType::Set)))
                        }
                        BinaryOp::Divide => {
                            (is_numeric_type(&left_type) && is_numeric_type(&right_type))
                                || matches!((&left_type, &right_type), (SemanticType::Builtin(BuiltinType::Set), SemanticType::Builtin(BuiltinType::Set)))
                        }
                        BinaryOp::Div | BinaryOp::Mod => is_integer_type(&left_type) && is_integer_type(&right_type),
                        BinaryOp::Or | BinaryOp::And => is_boolean_type(&left_type) && is_boolean_type(&right_type),
                        BinaryOp::Equal | BinaryOp::NotEqual => are_relation_compatible(&left_type, &right_type),
                        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                            are_ordered_relation_compatible(&left_type, &right_type)
                        }
                        BinaryOp::In => {
                            is_integer_type(&left_type)
                                && matches!(right_type, SemanticType::Builtin(BuiltinType::Set))
                        }
                        BinaryOp::Is => unreachable!("handled above"),
                    };
                    if !valid {
                        let (line, column) = expr_position(expr);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!(
                                "invalid operands for {}: {} and {}",
                                render_binary_op(*op),
                                render_semantic_type(&left_type),
                                render_semantic_type(&right_type)
                            ),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    fn validate_is_expression(
        &self,
        left: &Expr,
        right: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let (subject_symbol, subject_type) = match left {
            Expr::Designator(designator) => {
                let symbol = if designator.base.module.is_none() {
                    self.lookup_symbol(&designator.base.name, local_symbols)
                } else {
                    None
                };
                let ty = self.infer_expr_type(left, local_symbols, scope_type_names);
                (symbol, ty)
            }
            _ => (None, self.infer_expr_type(left, local_symbols, scope_type_names)),
        };

        let Some(subject_type) = subject_type else {
            return;
        };

        let Expr::Designator(designator) = right else {
            let (line, column) = expr_position(right);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                "IS right operand must be a type identifier".to_string(),
            ));
            return;
        };

        if !designator.selectors.is_empty() || !self.designator_denotes_type_name(designator, local_symbols) {
            let (line, column) = expr_position(right);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                "IS right operand must be a type identifier".to_string(),
            ));
            return;
        }

        self.validate_type_test_operands(
            TypeTestContext::IsExpr,
            subject_symbol,
            &subject_type,
            &designator.base,
            procedure_name,
            scope_type_names,
            local_symbols,
            expr_position(right),
            diagnostics,
        );
    }

    fn validate_type_test_operands(
        &self,
        context: TypeTestContext,
        subject_symbol: Option<&SemanticSymbol>,
        subject_type: &SemanticType,
        target_ident: &QualIdent,
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
        position: (usize, usize),
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let Some(static_record_type) = self.type_test_static_record_type(subject_symbol, subject_type, local_symbols) else {
            let (line, column) = position;
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                context.invalid_subject_message(),
            ));
            return;
        };

        let target_type = self.resolve_named_type(target_ident, scope_type_names);
        // CP §11.7: a type guard target may name either a record type
        // (when the subject is a record receiver/parameter) or a pointer-to-
        // record type (when the subject is a pointer). Both forms reduce to
        // the same extends-check on the underlying record.
        let resolved_target = self.resolve_named_type_one_level(&target_type, local_symbols);
        let target_pointee_record = match &resolved_target {
            SemanticType::Pointer { target, .. } if self.is_record_type(target, local_symbols) => {
                Some((**target).clone())
            }
            _ => None,
        };
        // Imported types are opaque — trust that the programmer named an extension record.
        let target_is_known_record = target_pointee_record.is_some()
            || self.is_record_type(&target_type, local_symbols)
            || matches!(target_type, SemanticType::Named { kind: NamedTypeKind::Imported, .. });
        if !target_is_known_record {
            let (line, column) = position;
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                context.invalid_target_message(&render_qualident(target_ident)),
            ));
            return;
        }

        // Pick the record-typed view of the target for the extends check:
        // if it's `POINTER TO Foo`, compare against `Foo`; otherwise compare
        // against the named record directly.
        let target_for_extends = target_pointee_record.unwrap_or_else(|| target_type.clone());

        // Skip the extends check for imported types — we have no definition to verify against.
        let either_imported = matches!(target_type, SemanticType::Named { kind: NamedTypeKind::Imported, .. })
            || matches!(static_record_type, SemanticType::Named { kind: NamedTypeKind::Imported, .. })
            || matches!(target_for_extends, SemanticType::Named { kind: NamedTypeKind::Imported, .. });
        // ANYPTR / ANYREC subjects narrow to any record — skip the
        // structural extends check.
        let subject_is_universal = matches!(
            static_record_type,
            SemanticType::Builtin(BuiltinType::AnyRec) | SemanticType::Builtin(BuiltinType::AnyPtr)
        );
        if !either_imported && !subject_is_universal
            && !self.record_type_extends(&target_for_extends, &static_record_type, local_symbols)
        {
            let (line, column) = position;
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                context.non_extension_message(
                    &render_qualident(target_ident),
                    &render_semantic_type(&static_record_type),
                ),
            ));
        }
    }

    fn type_test_static_record_type(
        &self,
        subject_symbol: Option<&SemanticSymbol>,
        subject_type: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> Option<SemanticType> {
        // Resolve cross-module / local type aliases before deciding what
        // kind of subject this is. Required so that `loc: Files.Locator`
        // (which holds `Named { module: Some("Files"), name: "Locator" }`
        // and resolves to `Pointer { target: Files.LocatorDesc }`) is
        // recognised as a pointer subject for type-guard purposes.
        let resolved = self.resolve_named_type_one_level(subject_type, local_symbols);
        match &resolved {
            // ANYPTR / ANYREC are universal supertypes — every
            // pointer-to-record (resp. record) narrows from them.
            // The extends check downstream sees AnyRec and accepts
            // any concrete record target (CP §11.7 allows the type
            // guard regardless).
            SemanticType::Builtin(BuiltinType::AnyPtr)
            | SemanticType::Builtin(BuiltinType::AnyRec) => {
                Some(SemanticType::Builtin(BuiltinType::AnyRec))
            }
            SemanticType::Pointer { target, .. } => {
                if self.is_record_type(target, local_symbols) {
                    Some((**target).clone())
                } else {
                    None
                }
            }
            _ if matches!(subject_symbol.map(|symbol| symbol.kind), Some(SymbolKind::Parameter | SymbolKind::Receiver))
                && (self.is_record_type(&resolved, local_symbols)
                    || matches!(resolved, SemanticType::Named { kind: NamedTypeKind::Imported, .. })) =>
            {
                Some(resolved)
            }
            _ => None,
        }
    }

    fn is_record_type(&self, ty: &SemanticType, local_symbols: &[SemanticSymbol]) -> bool {
        matches!(
            self.resolve_named_type_one_level(ty, local_symbols),
            SemanticType::Record { .. }
        )
    }

    fn record_type_extends(
        &self,
        candidate: &SemanticType,
        base: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        if self.semantic_types_match(candidate, base, local_symbols) {
            return true;
        }

        match self.resolve_named_type_one_level(candidate, local_symbols) {
            SemanticType::Record { base: Some(parent), .. } => {
                self.record_type_extends(&parent, base, local_symbols)
            }
            _ => false,
        }
    }

    fn method_signatures_match(&self, expected: &ProcedureType, actual: &ProcedureType) -> bool {
        self.method_result_types_match(
            expected.result_type.as_deref(),
            actual.result_type.as_deref(),
        )
            && expected.parameters.len() == actual.parameters.len()
            && expected
                .parameters
                .iter()
                .zip(&actual.parameters)
                .all(|(left, right)| left.mode == right.mode && left.ty == right.ty)
    }

    fn method_result_types_match(
        &self,
        expected: Option<&SemanticType>,
        actual: Option<&SemanticType>,
    ) -> bool {
        match (expected, actual) {
            (None, None) => true,
            (Some(expected), Some(actual)) if expected == actual => true,
            (Some(expected), Some(actual)) => {
                let expected = self.resolve_named_type_one_level(expected, &self.module_symbols);
                let actual = self.resolve_named_type_one_level(actual, &self.module_symbols);
                match (&expected, &actual) {
                    (
                        SemanticType::Pointer { target: expected_target, .. },
                        SemanticType::Pointer { target: actual_target, .. },
                    ) => self.record_type_extends(actual_target, expected_target, &self.module_symbols),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn semantic_types_match(
        &self,
        left: &SemanticType,
        right: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        if left == right {
            return true;
        }

        if matches!(left, SemanticType::Named { .. }) && matches!(right, SemanticType::Named { .. }) {
            return false;
        }

        let resolved_left = self.resolve_named_type_one_level(left, local_symbols);
        let resolved_right = self.resolve_named_type_one_level(right, local_symbols);
        if resolved_left == *left && resolved_right == *right {
            return false;
        }

        self.semantic_types_match(&resolved_left, &resolved_right, local_symbols)
    }

    fn resolve_named_type_one_level(
        &self,
        ty: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> SemanticType {
        resolve_named_type_alias(
            ty,
            local_symbols,
            &self.module_symbols,
            &self.imported_modules,
            &mut HashSet::new(),
        )
        .cloned()
        .unwrap_or_else(|| ty.clone())
    }

    /// Resolve a named type alias chain all the way to the first
    /// non-`Named` concrete type.  Used where we need to inspect the
    /// *kind* of a type (e.g. "is it a Record?") even when the name is
    /// an alias chain that crosses module boundaries.
    fn resolve_named_type_fully(
        &self,
        ty: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> SemanticType {
        let mut current = ty.clone();
        let mut steps = 0usize;
        loop {
            let resolved = self.resolve_named_type_one_level(&current, local_symbols);
            if resolved == current || steps > 32 {
                return current;
            }
            current = resolved;
            steps += 1;
        }
    }

    fn is_managed_pointer_type(&self, ty: &SemanticType, local_symbols: &[SemanticSymbol]) -> bool {
        match self.resolve_named_type_one_level(ty, local_symbols) {
            SemanticType::Pointer { untagged, .. } => !untagged,
            _ => false,
        }
    }

    fn designator_denotes_type_name(
        &self,
        designator: &Designator,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        if designator.base.module.is_some() {
            return true;
        }

        builtin_type_by_name(&designator.base.name).is_some()
            || self
                .lookup_symbol(&designator.base.name, local_symbols)
                .is_some_and(|symbol| symbol.kind == SymbolKind::Type)
    }

    fn qualident_denotes_type_name(
        &self,
        ident: &QualIdent,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        if ident.module.is_some() {
            return true;
        }

        builtin_type_by_name(&ident.name).is_some()
            || self
                .lookup_symbol(&ident.name, local_symbols)
                .is_some_and(|symbol| symbol.kind == SymbolKind::Type)
    }

    fn walk_designator_expressions(
        &self,
        designator: &Designator,
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        local_symbols: &[SemanticSymbol],
        resolutions: &mut Vec<SelectorResolution>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        for selector in &designator.selectors {
            match selector {
                Selector::Index(items) | Selector::Call(items) => {
                    for item in items {
                        self.walk_expr(
                            item,
                            procedure_name,
                            scope_type_names,
                            local_symbols,
                            resolutions,
                            diagnostics,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn validate_designator(
        &self,
        designator: &Designator,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) -> Option<SemanticType> {
        let designator = self.normalize_designator(designator, local_symbols);
        let current = if designator.base.module.is_none() {
            match designator.base.name.as_str() {
                "TRUE" | "FALSE" => Some(SemanticType::Builtin(BuiltinType::Boolean)),
                "INF" => Some(SemanticType::Builtin(BuiltinType::Real)),
                _ if builtin_proc_by_name(&designator.base.name).is_some() => {
                    Some(SemanticType::BuiltinProc(
                        builtin_proc_by_name(&designator.base.name).expect("builtin checked"),
                    ))
                }
                _ => self.lookup_symbol_type(&designator.base.name, local_symbols),
            }
        } else {
            self.resolve_system_builtin(designator.base.module.as_deref(), &designator.base.name)
                .map(SemanticType::BuiltinProc)
        };

        if current.is_none() && designator.base.module.is_none() {
            let (line, column) = designator_position(&designator);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("identifier {} is not declared", designator.base.name),
            ));
            return None;
        }

        // Module-qualified designator that isn't a SYSTEM builtin: when the
        // imported module's CP source has been loaded, verify the name is
        // actually exported by it. Without this check sema silently lets
        // unresolved cross-module references through, and codegen later
        // emits malformed IR (cf. "unsupported cast from PointerType to i64"
        // when a runtime native artifact happened to expose the name).
        if current.is_none() {
            if let Some(module_name) = designator.base.module.as_deref() {
                if let Some(symbols) = self.imported_modules.get(module_name) {
                    let found = symbols
                        .iter()
                        .any(|sym| sym.name == designator.base.name && sym.exported);
                    if !found {
                        let (line, column) = designator_position(&designator);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!(
                                "module {} has no exported declaration named {}",
                                module_name, designator.base.name
                            ),
                        ));
                        return None;
                    }
                }
            }
        }

        let mut current = current?;
        for selector in &designator.selectors {
            match self.validate_selector(
                &current,
                selector,
                &designator,
                procedure_name,
                local_symbols,
                scope_type_names,
                diagnostics,
            ) {
                Some(next) => current = next,
                None => return None,
            }
        }

        Some(current)
    }

    fn validate_selector(
        &self,
        base: &SemanticType,
        selector: &Selector,
        designator: &Designator,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) -> Option<SemanticType> {
        let (line, column) = designator_position(designator);
        match selector {
            Selector::Field(name) => match self.lookup_record_member(base, name, local_symbols) {
                Some(RecordMember::Field(ty)) => Some(ty),
                Some(RecordMember::Method(signature)) => Some(SemanticType::Procedure(signature)),
                None => {
                    // Decide between "field does not exist on a record we
                    // can see" (a real error) and "base is an opaque named
                    // type we can't introspect" (suppress, can't be sure).
                    //
                    // We can now resolve cross-module imported records via
                    // `imported_modules`, so every Named with kind
                    // `UserDefined` or `Imported` whose underlying form
                    // (after one alias hop, possibly through a pointer)
                    // is a `Record` qualifies as visible — a missing field
                    // there is a real diagnostic, not an unknown-record
                    // suppression.
                    if base_is_introspectable_record(
                        base,
                        local_symbols,
                        &self.module_symbols,
                        &self.imported_modules,
                    ) {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!(
                                "field {} does not exist on {}",
                                name,
                                render_semantic_type(base)
                            ),
                        ));
                        return None;
                    }
                    if matches!(
                        base,
                        SemanticType::Named { kind: NamedTypeKind::Unresolved, .. }
                    ) {
                        return None;
                    }
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!(
                            "field selector .{} requires a record, found {}",
                            name,
                            render_semantic_type(base)
                        ),
                    ));
                    None
                }
            },
            Selector::Index(items) => {
                for item in items {
                    if let Some(index_type) = self.infer_expr_type(item, local_symbols, scope_type_names) {
                        if !is_integer_type(&index_type) {
                            let (index_line, index_column) = expr_position(item);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                index_line,
                                index_column,
                                format!(
                                    "array index must be an integer, found {}",
                                    render_semantic_type(&index_type)
                                ),
                            ));
                        }
                    }
                }
                // Resolve Named aliases (e.g. `Bag = POINTER TO ARRAY OF
                // SHORTINT` — base shows up as `Named("Bag")`). Then
                // auto-dereference through pointer-to-array (CP §6.4
                // lets `p[i]` mean `p^[i]`).
                let resolved = self.resolve_named_type_one_level(base, local_symbols);
                let resolved = match &resolved {
                    SemanticType::Pointer { target, .. } => {
                        self.resolve_named_type_one_level(target, local_symbols)
                    }
                    _ => resolved,
                };
                match resolved {
                    SemanticType::Array { element_type, .. } => Some(*element_type),
                    _ => {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!("index selector requires an array, found {}", render_semantic_type(base)),
                        ));
                        None
                    }
                }
            }
            Selector::Dereference => {
                // Resolve Named aliases so `Bag^` works when
                // `Bag = POINTER TO ARRAY OF T` (the base's surface
                // type is `Named("Bag")`).
                let resolved = self.resolve_named_type_one_level(base, local_symbols);
                match resolved {
                    SemanticType::Pointer { target, .. } => Some(*target),
                    SemanticType::Procedure(sig) => {
                        // Super-call notation: v.M^ — dereference after a method name invokes the
                        // inherited (base) implementation.  Reject ABSTRACT / EMPTY targets.
                        if matches!(sig.flavor, Some(MethodFlavor::Abstract) | Some(MethodFlavor::Empty)) {
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                format!(
                                    "super-call is not permitted on {} methods",
                                    method_flavor_name(sig.flavor.unwrap()),
                                ),
                            ));
                        }
                        Some(base.clone())
                    }
                    // Imported / unresolved types may well be pointer types — suppress the error.
                    SemanticType::Named { kind: NamedTypeKind::Imported | NamedTypeKind::Unresolved, .. } => None,
                    _ => {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!("dereference requires a pointer, found {}", render_semantic_type(base)),
                        ));
                        None
                    }
                }
            },
            Selector::TypeGuard(guard) => self.validate_designator_type_guard(
                base,
                guard,
                designator,
                procedure_name,
                local_symbols,
                scope_type_names,
                diagnostics,
            ),
            Selector::AmbiguousParen(guard) => {
                // The base may be a Named alias of a procedure type
                // (`TYPE Op = PROCEDURE(...): T; VAR f: Op; f(...)`) —
                // unwrap one level so the call/guard disambiguation
                // sees the underlying Procedure.
                let base_unwrapped =
                    self.unwrap_named_aliases(base.clone(), local_symbols);
                if matches!(
                    base_unwrapped,
                    SemanticType::Procedure(_) | SemanticType::BuiltinProc(_)
                ) {
                    let synthetic_arg = Expr::Designator(Designator {
                        span: guard.span,
                        base: guard.clone(),
                        selectors: Vec::new(),
                    });
                    self.walk_expr(
                        &synthetic_arg,
                        procedure_name,
                        scope_type_names,
                        local_symbols,
                        &mut Vec::new(),
                        diagnostics,
                    );
                    match &base_unwrapped {
                        SemanticType::Procedure(signature) => {
                            self.validate_call_arguments(
                                signature,
                                &[synthetic_arg],
                                procedure_name,
                                local_symbols,
                                scope_type_names,
                                diagnostics,
                            );
                            signature.result_type.as_ref().map(|result| (**result).clone())
                        }
                        SemanticType::BuiltinProc(proc) => {
                            self.validate_builtin_call(
                                *proc,
                                &[synthetic_arg.clone()],
                                procedure_name,
                                local_symbols,
                                scope_type_names,
                                diagnostics,
                            );
                            builtin_proc_result_type(
                                *proc,
                                &[synthetic_arg],
                                local_symbols,
                                &self.module_symbols,
                                scope_type_names,
                            )
                        }
                        _ => None,
                    }
                } else if self.qualident_denotes_type_name(guard, local_symbols) {
                    self.validate_designator_type_guard(
                        base,
                        guard,
                        designator,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    )
                } else {
                    Some(self.resolve_named_type(guard, scope_type_names))
                }
            }
            Selector::Call(args) => match self
                .unwrap_named_aliases(base.clone(), local_symbols)
            {
                SemanticType::Procedure(signature) => {
                    self.validate_call_arguments(
                        &signature,
                        args,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                    signature.result_type.as_ref().map(|result| (**result).clone())
                }
                SemanticType::BuiltinProc(proc) => {
                    self.validate_builtin_call(
                        proc,
                        args,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                    builtin_proc_result_type(
                        proc,
                        args,
                        local_symbols,
                        &self.module_symbols,
                        scope_type_names,
                    )
                }
                // Named procedure-type alias (e.g. TYPE Handler = PROCEDURE(...)).
                // Resolve one level and retry so that a variable of a procedure type
                // is treated as callable.
                SemanticType::Named { .. } => {
                    let resolved = self.resolve_named_type_one_level(base, local_symbols);
                    if let SemanticType::Procedure(signature) = &resolved {
                        let signature = signature.clone();
                        self.validate_call_arguments(
                            &signature,
                            args,
                            procedure_name,
                            local_symbols,
                            scope_type_names,
                            diagnostics,
                        );
                        signature.result_type.as_ref().map(|result| (**result).clone())
                    } else {
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!("call selector requires a procedure, found {}", render_semantic_type(base)),
                        ));
                        None
                    }
                }
                _ => {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!("call selector requires a procedure, found {}", render_semantic_type(base)),
                    ));
                    None
                }
            },
            Selector::StringDereference => Some(SemanticType::Builtin(BuiltinType::Char)),
        }
    }

    fn validate_designator_type_guard(
        &self,
        base: &SemanticType,
        guard: &QualIdent,
        designator: &Designator,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) -> Option<SemanticType> {
        let designator = self.normalize_designator(designator, local_symbols);
        let subject_symbol = if designator.base.module.is_none() {
            self.lookup_symbol(&designator.base.name, local_symbols)
        } else {
            None
        };
        self.validate_type_test_operands(
            TypeTestContext::DesignatorGuard,
            subject_symbol,
            base,
            guard,
            procedure_name,
            scope_type_names,
            local_symbols,
            designator_position(&designator),
            diagnostics,
        );
        Some(self.resolve_named_type(guard, scope_type_names))
    }

    fn validate_call_arguments(
        &self,
        signature: &ProcedureType,
        args: &[Expr],
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let parameters = flatten_parameter_bindings(&signature.parameters);
        if parameters.len() != args.len() {
            let (line, column) = args
                .first()
                .map(expr_position)
                .unwrap_or((0, 0));
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!(
                    "procedure argument count mismatch: expected {}, found {}",
                    parameters.len(),
                    args.len()
                ),
            ));
        }

        for (index, (expected, actual_expr)) in parameters.iter().zip(args.iter()).enumerate() {
            if matches!(expected.mode, Some(ParamMode::Var) | Some(ParamMode::Out)) {
                self.validate_argument_mode(
                    index,
                    expected,
                    actual_expr,
                    procedure_name,
                    local_symbols,
                    diagnostics,
                );
            }

            if let Some(actual) = self.infer_expr_type(actual_expr, local_symbols, scope_type_names) {
                // CP §8.1: a VAR / IN / OUT parameter of record type
                // accepts any extension of that record type.  The
                // formal binds to the actual's storage in place, so
                // there's no truncation hazard — the callee just sees
                // the base subobject view.  We unwrap one level of
                // named-alias on both sides so cross-module record
                // names (e.g. Sequencers.Message) hit the Record arm.
                let record_extension_ok = matches!(
                    expected.mode,
                    Some(ParamMode::Var) | Some(ParamMode::In) | Some(ParamMode::Out)
                ) && {
                    // Resolve the expected type FULLY (follow alias chains
                    // across module boundaries) so that a type alias such as
                    // `Message* = Views.CtrlMessage` correctly resolves to
                    // the underlying Record and not to a Named stub.
                    let exp = self.resolve_named_type_fully(&expected.ty, local_symbols);
                    let act = self.resolve_named_type_one_level(&actual, local_symbols);
                    matches!(exp, SemanticType::Record { .. })
                        && matches!(act, SemanticType::Record { .. })
                        && self.record_type_extends(&actual, &expected.ty, local_symbols)
                };
                let compatible = self.types_are_assignment_compatible(&expected.ty, &actual, local_symbols)
                    || integer_literal_fits_target(actual_expr, &expected.ty)
                    || record_extension_ok;
                if !compatible {
                    let (line, column) = expr_position(actual_expr);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!(
                            "argument {} type mismatch: expected {}, found {}",
                            index + 1,
                            render_semantic_type(&expected.ty),
                            render_semantic_type(&actual)
                        ),
                    ));
                }
            }
        }
    }

    fn validate_argument_mode(
        &self,
        index: usize,
        expected: &ParameterBinding,
        actual_expr: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let Some(mode) = expected.mode else {
            return;
        };

        let Expr::Designator(designator) = actual_expr else {
            let (line, column) = expr_position(actual_expr);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!(
                    "argument {} for {} parameter must be an assignable designator",
                    index + 1,
                    render_param_mode(mode)
                ),
            ));
            return;
        };

        let designator = self.normalize_designator(designator, local_symbols);

        if designator
            .selectors
            .iter()
            .any(|selector| matches!(selector, Selector::Call(_) | Selector::TypeGuard(_) | Selector::AmbiguousParen(_)))
        {
            let (line, column) = designator_position(&designator);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!(
                    "argument {} for {} parameter is not assignable",
                    index + 1,
                    render_param_mode(mode)
                ),
            ));
            return;
        }

        if designator.base.module.is_some() {
            return;
        }

        if let Some(symbol) = self.lookup_symbol(&designator.base.name, local_symbols) {
            if !matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Parameter | SymbolKind::Receiver) {
                let (line, column) = designator_position(&designator);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "argument {} for {} parameter must name a variable, parameter, or receiver, found {} {}",
                        index + 1,
                        render_param_mode(mode),
                        render_symbol_kind(symbol.kind),
                        symbol.name
                    ),
                ));
            }
        }
    }

    fn validate_procedure_call_statement(
        &self,
        designator: &Designator,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let final_type =
            self.validate_designator(designator, procedure_name, local_symbols, scope_type_names, diagnostics);
        let has_explicit_call = designator
            .selectors
            .iter()
            .any(|selector| matches!(selector, Selector::Call(_)));

        if has_explicit_call {
            return;
        }

        if designator.base.module.is_none()
            && builtin_proc_by_name(&designator.base.name).is_some()
            && designator.selectors.is_empty()
        {
            let (line, column) = designator_position(designator);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                "procedure call is missing required arguments".to_string(),
            ));
            return;
        }

        if self
            .resolve_system_builtin(designator.base.module.as_deref(), &designator.base.name)
            .is_some()
            && designator.selectors.is_empty()
        {
            let (line, column) = designator_position(designator);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                "procedure call is missing required arguments".to_string(),
            ));
            return;
        }

        match final_type {
            Some(SemanticType::Procedure(signature)) => {
                if !signature.parameters.is_empty() {
                    let (line, column) = designator_position(designator);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        "procedure call is missing required arguments".to_string(),
                    ));
                }
            }
            Some(SemanticType::BuiltinProc(_)) => {}
            Some(other) => {
                let (line, column) = designator_position(designator);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "procedure call statement requires a procedure designator, found {}",
                        render_semantic_type(&other)
                    ),
                ));
            }
            None => {}
        }
    }

    fn apply_selector_type(
        &self,
        base: &SemanticType,
        selector: &Selector,
        scope_type_names: &HashSet<String>,
    ) -> Option<SemanticType> {
        match selector {
            Selector::Field(name) => self.lookup_record_member(base, name, &[]).map(|member| match member {
                RecordMember::Field(ty) => ty,
                RecordMember::Method(signature) => SemanticType::Procedure(signature),
            }),
            Selector::Index(_) => match base {
                SemanticType::Array { element_type, .. } => Some((**element_type).clone()),
                _ => None,
            },
            Selector::Dereference => match base {
                SemanticType::Pointer { target, .. } => Some((**target).clone()),
                // Super-call passthrough: v.M^ yields the same procedure type
                SemanticType::Procedure(_) => Some(base.clone()),
                // Imported / unresolved named types may be pointers — don't reject them
                SemanticType::Named { kind: NamedTypeKind::Imported | NamedTypeKind::Unresolved, .. } => None,
                _ => None,
            },
            Selector::TypeGuard(guard) => {
                Some(self.resolve_named_type(guard, scope_type_names))
            }
            Selector::AmbiguousParen(guard) => {
                // Unwrap Named-aliased procedure types (`TYPE Op =
                // PROCEDURE(...)`) so a parameter declared with the
                // alias is recognised as callable, not type-guard'd.
                let base_unwrapped = self
                    .unwrap_named_aliases(base.clone(), &[]);
                match base_unwrapped {
                    SemanticType::Procedure(signature) => {
                        signature.result_type.as_ref().map(|result| (**result).clone())
                    }
                    SemanticType::BuiltinProc(proc) => builtin_proc_result_type(
                        proc,
                        &[Expr::Designator(Designator {
                            span: guard.span,
                            base: guard.clone(),
                            selectors: Vec::new(),
                        })],
                        &[],
                        &self.module_symbols,
                        scope_type_names,
                    ),
                    _ => Some(self.resolve_named_type(guard, scope_type_names)),
                }
            }
            Selector::Call(_) => match self.unwrap_named_aliases(base.clone(), &[]) {
                SemanticType::Procedure(signature) => {
                    signature.result_type.as_ref().map(|result| (**result).clone())
                }
                SemanticType::BuiltinProc(proc) => {
                    builtin_proc_result_type(proc, &[], &[], &self.module_symbols, scope_type_names)
                }
                _ => None,
            },
            Selector::StringDereference => Some(SemanticType::Builtin(BuiltinType::Char)),
        }
    }

    fn lookup_symbol_type(
        &self,
        name: &str,
        local_symbols: &[SemanticSymbol],
    ) -> Option<SemanticType> {
        self.lookup_symbol(name, local_symbols)
            .and_then(|symbol| symbol.declared_type.clone())
    }

    fn lookup_symbol<'b>(
        &'b self,
        name: &str,
        local_symbols: &'b [SemanticSymbol],
    ) -> Option<&'b SemanticSymbol> {
        local_symbols
            .iter()
            .rev()
            .chain(self.module_symbols.iter().rev())
            .find(|symbol| symbol.name == name)
    }

    /// CP §11.5 receiver-return-type compatibility check.
    ///
    /// Accept `RETURN <receiver>` from a method whose declared return
    /// type is the receiver's pointer alias, even though sema
    /// canonicalises the receiver to the underlying record (so dispatch
    /// matching works uniformly across `(b: Box)` and `(b: BoxDesc)`).
    /// Without this, code like
    /// ```cp
    /// PROCEDURE (b: Box) WithValue (n: INTEGER): Box, NEW;
    /// BEGIN
    ///     b.v := n;
    ///     RETURN b
    /// END WithValue;
    /// ```
    /// trips with `expected Box, found BoxDesc`.
    ///
    /// Returns `true` if the return expression is a bare designator
    /// naming a receiver formal AND the expected type (after alias
    /// unwrap) is `Pointer { target = Named(R) }` where R resolves
    /// to the same Record `actual` is.
    fn return_is_receiver_pointer_round_trip(
        &self,
        expr: &Expr,
        expected: &SemanticType,
        actual: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        let Expr::Designator(des) = expr else { return false; };
        if des.base.module.is_some() || !des.selectors.is_empty() {
            return false;
        }
        let Some(sym) = local_symbols.iter().find(|s| s.name == des.base.name) else {
            return false;
        };
        if sym.kind != SymbolKind::Receiver {
            return false;
        }
        // Unwrap both sides through any named-alias chain so we're
        // comparing the underlying structural shapes.  `actual` lands
        // as a Record (the receiver's canonical record); `expected`
        // lands as a Pointer whose target resolves to the same Record.
        let actual_unwrapped =
            self.unwrap_named_aliases(actual.clone(), local_symbols);
        if !matches!(actual_unwrapped, SemanticType::Record { .. }) {
            return false;
        }
        let expected_unwrapped =
            self.unwrap_named_aliases(expected.clone(), local_symbols);
        let SemanticType::Pointer { target, .. } = expected_unwrapped else {
            return false;
        };
        let target_resolved =
            self.unwrap_named_aliases((*target).clone(), local_symbols);
        target_resolved == actual_unwrapped
    }

    fn normalize_designator(
        &self,
        designator: &Designator,
        local_symbols: &[SemanticSymbol],
    ) -> Designator {
        let Some(module_name) = designator.base.module.as_ref() else {
            return designator.clone();
        };
        let Some(symbol) = self.lookup_symbol(module_name, local_symbols) else {
            return designator.clone();
        };
        if matches!(symbol.kind, SymbolKind::Import | SymbolKind::Type) {
            return designator.clone();
        }

        let mut normalized = designator.clone();
        let field_name = normalized.base.name.clone();
        normalized.base.name = module_name.clone();
        normalized.base.module = None;
        normalized.selectors.insert(0, Selector::Field(field_name));
        normalized
    }

    fn types_are_assignment_compatible(
        &self,
        expected: &SemanticType,
        actual: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        // Unwrap alias chains fully on both sides — a cross-module
        // alias like `Stores.Store = INTEGER` needs at least two
        // hops (imported Named → INTEGER → Builtin(Integer)) before
        // we can compare structurally against an INTEGER target.
        let expected = self.unwrap_named_aliases(expected.clone(), local_symbols);
        let actual = self.unwrap_named_aliases(actual.clone(), local_symbols);

        if expected == actual {
            return true;
        }

        if matches!(actual, SemanticType::Nil)
            && matches!(expected, SemanticType::Pointer { .. } | SemanticType::Procedure(_))
        {
            return true;
        }

        // ANYPTR is the universal pointer type — accepts NIL, any
        // pointer-typed value, any procedure value.  CP §6.4: ANYPTR
        // is the supertype of every pointer type; assignment from a
        // typed pointer to ANYPTR is implicit.
        if matches!(expected, SemanticType::Builtin(BuiltinType::AnyPtr))
            && matches!(
                actual,
                SemanticType::Pointer { .. }
                    | SemanticType::Procedure(_)
                    | SemanticType::Nil
                    | SemanticType::Builtin(BuiltinType::AnyPtr)
            )
        {
            return true;
        }
        // The reverse — narrowing from ANYPTR to a typed pointer —
        // is NOT implicit; CP requires a type guard (`p(SomePtr)`)
        // or WITH for that.  No automatic acceptance here.

        // ANYREC is the universal record type; in a VAR/IN/OUT param
        // position any record extension flows through (mirrors the
        // record-extension widening already covered for typed bases).
        if matches!(expected, SemanticType::Builtin(BuiltinType::AnyRec)) {
            let actual_resolved = self.resolve_named_type_one_level(&actual, local_symbols);
            if matches!(actual_resolved, SemanticType::Record { .. })
                || matches!(actual, SemanticType::Builtin(BuiltinType::AnyRec))
            {
                return true;
            }
        }

        if is_numeric_type(&expected) && is_numeric_type(&actual) {
            return numeric_rank(&actual) <= numeric_rank(&expected);
        }

        if is_character_like_type(&expected) && is_character_like_type(&actual) {
            return character_rank(&actual) <= character_rank(&expected);
        }

        if matches!(
            actual,
            SemanticType::Builtin(BuiltinType::ShortString) | SemanticType::Builtin(BuiltinType::String)
        ) {
            let accepts_char_array = match &expected {
                SemanticType::Array { element_type, .. } => matches!(
                    element_type.as_ref(),
                    SemanticType::Builtin(BuiltinType::Char)
                        | SemanticType::Builtin(BuiltinType::ShortChar)
                ),
                _ => false,
            };
            let requires_shortstring = match &expected {
                SemanticType::Array { element_type, .. } => {
                    !matches!(element_type.as_ref(), SemanticType::Builtin(BuiltinType::Char))
                }
                _ => false,
            };
            return accepts_char_array
                && (!requires_shortstring
                    || matches!(actual, SemanticType::Builtin(BuiltinType::ShortString)));
        }

        match (&expected, &actual) {
            (SemanticType::Pointer { target: expected_target, .. }, SemanticType::Pointer { target: actual_target, .. }) => {
                self.record_type_extends(actual_target, expected_target, local_symbols)
            }
            (SemanticType::Procedure(expected_sig), SemanticType::Procedure(actual_sig)) => {
                procedure_types_match(expected_sig, actual_sig)
            }
            // Open array parameter (lengths empty) accepts any concrete array with a
            // compatible element type.  This is the standard Oberon/CP rule:
            //   ARRAY OF T  ← ARRAY n OF T  (for any positive n)
            (
                SemanticType::Array { lengths: expected_lengths, element_type: expected_elem, .. },
                SemanticType::Array { element_type: actual_elem, .. },
            ) if expected_lengths.is_empty() => {
                self.types_are_assignment_compatible(expected_elem, actual_elem, local_symbols)
            }
            _ => false,
        }
    }

    fn lookup_record_member(
        &self,
        ty: &SemanticType,
        name: &str,
        local_symbols: &[SemanticSymbol],
    ) -> Option<RecordMember> {
        match self.resolve_named_type_one_level(ty, local_symbols) {
            SemanticType::Record {
                fields,
                methods,
                base,
                ..
            } => {
                if let Some(field) = fields
                    .iter()
                    .find(|field| field.names.iter().any(|field_name| field_name == name))
                {
                    return Some(RecordMember::Field(field.ty.clone()));
                }
                if let Some(method) = methods.iter().find(|method| method.name == name) {
                    return Some(RecordMember::Method(method.signature.clone()));
                }
                base.as_ref()
                    .and_then(|parent| self.lookup_record_member(parent, name, local_symbols))
            }
            // CP §6.4: a pointer designator denotes the referenced record;
            // `p.f` is shorthand for `p^.f`. Recurse through the pointee so
            // pointer-aliased receivers, vars, and parameters all work.
            SemanticType::Pointer { target, .. } => {
                self.lookup_record_member(&target, name, local_symbols)
            }
            _ => None,
        }
    }

    fn resolve_system_builtin(&self, module_qualifier: Option<&str>, name: &str) -> Option<BuiltinProc> {
        let qualifier = module_qualifier?;
        if !self.has_system_import {
            return None;
        }
        let matches_system = self.module.imports.iter().any(|item| {
            item.name == "SYSTEM"
                && (item.name == qualifier || item.alias.as_deref() == Some(qualifier))
        });
        if !matches_system {
            return None;
        }

        match name {
            "ADR" => Some(BuiltinProc::SystemAdr),
            "VAL" => Some(BuiltinProc::SystemVal),
            "LSH" => Some(BuiltinProc::SystemLsh),
            "ROT" => Some(BuiltinProc::SystemRot),
            "TYP" => Some(BuiltinProc::SystemTyp),
            "BIT" => Some(BuiltinProc::SystemBit),
            "GET" => Some(BuiltinProc::SystemGet),
            "PUT" => Some(BuiltinProc::SystemPut),
            "MOVE" => Some(BuiltinProc::SystemMove),
            "NEW" => Some(BuiltinProc::SystemNew),
            "GETREG" => Some(BuiltinProc::SystemGetReg),
            "PUTREG" => Some(BuiltinProc::SystemPutReg),
            _ => None,
        }
    }

    fn record_designator_resolution(
        &self,
        designator: &Designator,
        procedure_name: Option<&str>,
        scope_type_names: &HashSet<String>,
        resolutions: &mut Vec<SelectorResolution>,
    ) {
        let rendered = render_designator(designator);
        for selector in &designator.selectors {
            match selector {
                Selector::AmbiguousParen(qualident) => {
                    // If the base of the designator is a procedure (builtin or user-defined),
                    // the single-identifier argument is a call argument, not a type guard.
                    let base_is_procedure = self.module_symbols.iter().any(|s| {
                        s.name == designator.base.name
                            && matches!(s.kind, SymbolKind::Procedure)
                    });
                    let (kind, reason) = if base_is_procedure {
                        (
                            SelectorResolutionKind::ProcedureCall,
                            format!(
                                "{} is a procedure; single-identifier argument is a call arg",
                                designator.base.name
                            ),
                        )
                    } else {
                        let resolved_type = self.resolve_named_type(qualident, scope_type_names);
                        (
                            selector_resolution_kind_for_type(&resolved_type),
                            format!(
                                "ambiguous parenthesized selector resolves as {}",
                                render_semantic_type(&resolved_type)
                            ),
                        )
                    };
                    resolutions.push(SelectorResolution {
                        procedure: procedure_name.map(str::to_string),
                        designator: rendered.clone(),
                        selector: render_qualident(qualident),
                        kind,
                        reason,
                    });
                }
                Selector::TypeGuard(qualident) => resolutions.push(SelectorResolution {
                    procedure: procedure_name.map(str::to_string),
                    designator: rendered.clone(),
                    selector: render_qualident(qualident),
                    kind: SelectorResolutionKind::TypeGuard,
                    reason: "parser recognized explicit type-guard selector".to_string(),
                }),
                Selector::Call(_) => resolutions.push(SelectorResolution {
                    procedure: procedure_name.map(str::to_string),
                    designator: rendered.clone(),
                    selector: "()".to_string(),
                    kind: SelectorResolutionKind::ProcedureCall,
                    reason: "designator contains an argument list".to_string(),
                }),
                _ => {}
            }
        }
    }
}

fn builtin_symbols() -> Vec<SemanticSymbol> {
    let mut symbols = builtin_types()
        .iter()
        .map(|builtin| SemanticSymbol {
            name: builtin.name().to_string(),
            kind: SymbolKind::Type,
            exported: false,
            read_only_export: false,
            declared_type: Some(SemanticType::Builtin(*builtin)),
            const_value: None,
            simd_shape: None,
            param_mode: None,
        })
        .collect::<Vec<_>>();

    symbols.extend([
        SemanticSymbol {
            name: "TRUE".to_string(),
            kind: SymbolKind::Constant,
            exported: false,
            read_only_export: false,
            declared_type: Some(SemanticType::Builtin(BuiltinType::Boolean)),
            const_value: Some(ConstValue::Boolean(true)),
            simd_shape: None,
            param_mode: None,
        },
        SemanticSymbol {
            name: "FALSE".to_string(),
            kind: SymbolKind::Constant,
            exported: false,
            read_only_export: false,
            declared_type: Some(SemanticType::Builtin(BuiltinType::Boolean)),
            const_value: Some(ConstValue::Boolean(false)),
            simd_shape: None,
            param_mode: None,
        },
        SemanticSymbol {
            name: "INF".to_string(),
            kind: SymbolKind::Constant,
            exported: false,
            read_only_export: false,
            declared_type: Some(SemanticType::Builtin(BuiltinType::Real)),
            const_value: None,
            simd_shape: None,
            param_mode: None,
        },
    ]);

    symbols.extend(builtin_procs().iter().map(|builtin| SemanticSymbol {
        name: builtin.name().to_string(),
        kind: SymbolKind::Procedure,
        exported: false,
        read_only_export: false,
        declared_type: Some(SemanticType::BuiltinProc(*builtin)),
        const_value: None,
        simd_shape: None,
        param_mode: None,
    }));

    symbols
}

/// Walk a list of statements and collect all unqualified identifier names referenced.
/// Used to determine which outer-scope variables a nested procedure captures.
fn collect_free_names_in_stmts(stmts: &[newcp_parser::Statement]) -> HashSet<String> {
    use newcp_parser::{Expr, Selector, Statement};
    let mut names = HashSet::new();

    fn walk_expr(expr: &Expr, names: &mut HashSet<String>) {
        match expr {
            Expr::Designator(des) => {
                // The parser eagerly folds `x.y` into a module-prefixed
                // QualIdent when `x` could lexically be a module name —
                // so for `f.unit` (where `f` is a record-typed local),
                // the QualIdent ends up as `(module: Some("f"), name: "unit")`.
                // The free-name walk has to consider both halves: register
                // `name` when no module prefix, otherwise register the
                // prefix as the actual free reference.
                if des.base.module.is_none() {
                    names.insert(des.base.name.clone());
                } else if let Some(m) = &des.base.module {
                    names.insert(m.clone());
                }
                for sel in &des.selectors {
                    match sel {
                        Selector::Call(args) => {
                            for a in args { walk_expr(a, names); }
                        }
                        Selector::Index(indices) => {
                            for i in indices { walk_expr(i, names); }
                        }
                        Selector::AmbiguousParen(qi) => {
                            if qi.module.is_none() { names.insert(qi.name.clone()); }
                        }
                        _ => {}
                    }
                }
            }
            Expr::Unary { expr, .. } => walk_expr(expr, names),
            Expr::Binary { left, right, .. } => {
                walk_expr(left, names);
                walk_expr(right, names);
            }
            Expr::Set { elements, .. } => {
                for e in elements {
                    walk_expr(&e.start, names);
                    if let Some(end) = &e.end { walk_expr(end, names); }
                }
            }
            _ => {}
        }
    }

    fn walk_stmts(stmts: &[newcp_parser::Statement], names: &mut HashSet<String>) {
        for stmt in stmts {
            match stmt {
                Statement::Assignment { target, value, .. } => {
                    if target.base.module.is_none() { names.insert(target.base.name.clone()); }
                    walk_expr(value, names);
                }
                Statement::ProcedureCall { designator, .. } => {
                    if designator.base.module.is_none() { names.insert(designator.base.name.clone()); }
                    for sel in &designator.selectors {
                        if let Selector::Call(args) = sel {
                            for a in args { walk_expr(a, names); }
                        }
                    }
                }
                Statement::Return { expr, .. } => {
                    if let Some(e) = expr { walk_expr(e, names); }
                }
                Statement::If { branches, else_branch, .. } => {
                    for b in branches {
                        walk_expr(&b.condition, names);
                        walk_stmts(&b.body, names);
                    }
                    if let Some(eb) = else_branch { walk_stmts(eb, names); }
                }
                Statement::While { condition, body, .. } => {
                    walk_expr(condition, names);
                    walk_stmts(body, names);
                }
                Statement::Repeat { body, until, .. } => {
                    walk_stmts(body, names);
                    walk_expr(until, names);
                }
                Statement::For { variable, start, end, step, body, .. } => {
                    names.insert(variable.clone());
                    walk_expr(start, names);
                    walk_expr(end, names);
                    if let Some(s) = step { walk_expr(s, names); }
                    walk_stmts(body, names);
                }
                Statement::Loop { body, .. } => walk_stmts(body, names),
                Statement::Case { expr, arms, else_branch, .. } => {
                    walk_expr(expr, names);
                    for arm in arms { walk_stmts(&arm.body, names); }
                    if let Some(eb) = else_branch { walk_stmts(eb, names); }
                }
                Statement::With { arms, else_branch, .. } => {
                    for arm in arms { walk_stmts(&arm.body, names); }
                    if let Some(eb) = else_branch { walk_stmts(eb, names); }
                }
                Statement::Empty { .. } | Statement::Exit { .. } | Statement::Brk { .. } => {}
            }
        }
    }

    walk_stmts(stmts, &mut names);
    names
}

fn annotate_simd_shapes(symbols: &mut [SemanticSymbol], outer_symbols: &[SemanticSymbol]) {
    let snapshot = symbols.to_vec();
    // SIMD shape inference for cross-module typedefs is intentionally skipped:
    // the SIMD-eligible types (packed records of f32/i64/etc.) live in the
    // module being analysed, not as imports. Pass an empty map to keep the
    // helpers' signature consistent with the rest of the alias-resolver path.
    let empty_imports: HashMap<String, Vec<SemanticSymbol>> = HashMap::new();
    for symbol in symbols.iter_mut() {
        symbol.simd_shape = symbol
            .declared_type
            .as_ref()
            .and_then(|ty| infer_simd_shape(ty, &snapshot, outer_symbols, &empty_imports, &mut HashSet::new()));
    }
}

fn infer_simd_shape(
    ty: &SemanticType,
    local_symbols: &[SemanticSymbol],
    outer_symbols: &[SemanticSymbol],
    imported_modules: &HashMap<String, Vec<SemanticSymbol>>,
    seen_named: &mut HashSet<String>,
) -> Option<SimdShape> {
    match resolve_named_type_alias(ty, local_symbols, outer_symbols, imported_modules, seen_named).unwrap_or(ty) {
        SemanticType::Record {
            base,
            flavor,
            fields,
            methods,
            ..
        } if base.is_none() && flavor.is_none() && methods.is_empty() => {
            let (lane_kind, lane_count) = infer_homogeneous_record_lanes(
                fields,
                local_symbols,
                outer_symbols,
                imported_modules,
                seen_named,
            )?;
            Some(make_simd_shape(
                SimdLayout::PackedRecord,
                lane_kind,
                lane_count,
            ))
        }
        SemanticType::Array { element_type, .. } => {
            if let Some(lane_kind) =
                resolve_simd_scalar_lane(element_type, local_symbols, outer_symbols, imported_modules, seen_named)
            {
                return Some(make_simd_shape(SimdLayout::ScalarArray, lane_kind, 1));
            }
            match infer_simd_shape(element_type, local_symbols, outer_symbols, imported_modules, seen_named)? {
                SimdShape {
                    layout: SimdLayout::PackedRecord,
                    lane_kind,
                    lane_count,
                    ..
                } => Some(make_simd_shape(SimdLayout::ArrayOfStruct, lane_kind, lane_count)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn make_simd_shape(layout: SimdLayout, lane_kind: SimdLaneKind, lane_count: usize) -> SimdShape {
    SimdShape {
        layout,
        lane_kind,
        lane_count,
        packed_bytes: lane_kind.lane_bytes() * lane_count,
    }
}

fn infer_homogeneous_record_lanes(
    fields: &[FieldType],
    local_symbols: &[SemanticSymbol],
    outer_symbols: &[SemanticSymbol],
    imported_modules: &HashMap<String, Vec<SemanticSymbol>>,
    seen_named: &mut HashSet<String>,
) -> Option<(SimdLaneKind, usize)> {
    let mut lane_kind = None;
    let mut lane_count = 0usize;

    for field in fields {
        let field_lane = resolve_simd_scalar_lane(&field.ty, local_symbols, outer_symbols, imported_modules, seen_named)?;
        if let Some(existing) = lane_kind {
            if existing != field_lane {
                return None;
            }
        } else {
            lane_kind = Some(field_lane);
        }
        lane_count += field.names.len();
    }

    let lane_kind = lane_kind?;
    if is_simd_lane_count(lane_count) {
        Some((lane_kind, lane_count))
    } else {
        None
    }
}

fn resolve_simd_scalar_lane(
    ty: &SemanticType,
    local_symbols: &[SemanticSymbol],
    outer_symbols: &[SemanticSymbol],
    imported_modules: &HashMap<String, Vec<SemanticSymbol>>,
    seen_named: &mut HashSet<String>,
) -> Option<SimdLaneKind> {
    match resolve_named_type_alias(ty, local_symbols, outer_symbols, imported_modules, seen_named).unwrap_or(ty) {
        SemanticType::Builtin(BuiltinType::ShortReal) => Some(SimdLaneKind::Float32),
        SemanticType::Builtin(BuiltinType::Real) => Some(SimdLaneKind::Float64),
        SemanticType::Builtin(BuiltinType::IntShort) => Some(SimdLaneKind::Int32),
        SemanticType::Builtin(BuiltinType::Integer)
        | SemanticType::Builtin(BuiltinType::LongInt) => Some(SimdLaneKind::Int64),
        _ => None,
    }
}

/// Does `ty` reach a `Record` (after at most one alias hop and at most
/// one pointer dereference)?  Used by the missing-field diagnostic to
/// decide between "the record exists, you mis-spelled the field" and
/// "the base is opaque to us, suppress".  Now that imported modules'
/// symbol tables are populated, every cross-module record with a CP
/// source counts as introspectable.
fn base_is_introspectable_record(
    ty: &SemanticType,
    local_symbols: &[SemanticSymbol],
    module_symbols: &[SemanticSymbol],
    imported_modules: &HashMap<String, Vec<SemanticSymbol>>,
) -> bool {
    let mut seen: HashSet<String> = HashSet::new();
    let resolved = resolve_named_type_alias(
        ty,
        local_symbols,
        module_symbols,
        imported_modules,
        &mut seen,
    )
    .cloned()
    .unwrap_or_else(|| ty.clone());
    match &resolved {
        SemanticType::Record { .. } => true,
        SemanticType::Pointer { target, .. } => {
            let mut seen: HashSet<String> = HashSet::new();
            let inner = resolve_named_type_alias(
                target,
                local_symbols,
                module_symbols,
                imported_modules,
                &mut seen,
            )
            .cloned()
            .unwrap_or_else(|| (**target).clone());
            matches!(inner, SemanticType::Record { .. })
        }
        _ => false,
    }
}

fn resolve_named_type_alias<'a>(
    ty: &'a SemanticType,
    local_symbols: &'a [SemanticSymbol],
    outer_symbols: &'a [SemanticSymbol],
    imported_modules: &'a HashMap<String, Vec<SemanticSymbol>>,
    seen_named: &mut HashSet<String>,
) -> Option<&'a SemanticType> {
    match ty {
        SemanticType::Named {
            module: None,
            name,
            kind: NamedTypeKind::UserDefined,
        } => {
            if !seen_named.insert(name.clone()) {
                return None;
            }
            let resolved = local_symbols
                .iter()
                .chain(outer_symbols.iter())
                .find(|symbol| symbol.kind == SymbolKind::Type && symbol.name == *name)
                .and_then(|symbol| symbol.declared_type.as_ref());
            seen_named.remove(name);
            resolved
        }
        SemanticType::Named {
            module: Some(module_name),
            name,
            kind: NamedTypeKind::UserDefined | NamedTypeKind::Imported,
        } => {
            // Cross-module type alias: look the type up in the imported
            // module's symbol table. Required so checks like "open ARRAY OF
            // CHAR accepts Files.Name (= ARRAY 16 OF CHAR)" succeed —
            // without this the check sees `imported:Files.Name` as an
            // opaque named type and fails.
            let key = format!("{module_name}::{name}");
            if !seen_named.insert(key.clone()) {
                return None;
            }
            let resolved = imported_modules
                .get(module_name)
                .and_then(|symbols| {
                    symbols
                        .iter()
                        .find(|symbol| {
                            symbol.kind == SymbolKind::Type
                                && symbol.name == *name
                                && symbol.exported
                        })
                        .and_then(|symbol| symbol.declared_type.as_ref())
                });
            seen_named.remove(&key);
            resolved
        }
        _ => None,
    }
}

/// Unwrap a possibly-pointer-aliased semantic type down to its
/// underlying Record. Used to inspect imported record types when
/// walking inheritance chains across modules (the same module's records
/// are inspected directly through `module.declarations`).
fn unwrap_to_record(ty: &SemanticType) -> Option<&SemanticType> {
    match ty {
        SemanticType::Record { .. } => Some(ty),
        SemanticType::Pointer { target, .. } => unwrap_to_record(target),
        _ => None,
    }
}

/// Read `Mod/<import>.cp` for each import in `module` and return
/// `(import_name, top-level symbols of that module)` pairs.
///
/// On any read or parse failure the import is silently omitted; downstream
/// code then treats that import as opaque (the same behaviour the sema
/// layer had before cross-module resolution existed). This keeps the
/// addition non-breaking for tests / fixtures that don't ship sources for
/// every import (e.g. the resident-module facades like SYSTEM, Console,
/// Math, …).
fn load_imported_module_symbols(
    module: &ModuleAst,
    source_dir: Option<&Path>,
) -> HashMap<String, Vec<SemanticSymbol>> {
    let mut out: HashMap<String, Vec<SemanticSymbol>> = HashMap::new();
    let debug = std::env::var("NEWCP_SEMA_DEBUG").is_ok();
    for import in &module.imports {
        if import.name == "SYSTEM" {
            // SYSTEM is provided by the analyzer's builtins, not as a
            // standalone source module.
            continue;
        }
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(dir) = source_dir {
            // Sibling of the importing module.
            candidates.push(dir.join(format!("{}.cp", import.name)));
            // Sibling Mod/ directory under the importing module's parent.
            // Handles fixtures in `Mod/Tests/` importing modules in `Mod/`.
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join(format!("{}.cp", import.name)));
            }
        }
        // Cwd-relative fallbacks (matches the IR layer's lookup behaviour).
        candidates.push(PathBuf::from("Mod").join(format!("{}.cp", import.name)));
        candidates.push(PathBuf::from(format!("{}.cp", import.name)));

        let chosen = candidates
            .iter()
            .find_map(|path| read_module_ast(path).ok().map(|ast| (path.clone(), ast)));
        let Some((found_path, ast)) = chosen else {
            if debug {
                eprintln!(
                    "[sema] import {} for module {} — no source found in any of {:?}",
                    import.name, module.name, candidates
                );
            }
            continue;
        };
        // Recursive analysis is bounded: CP forbids cyclic imports.
        // Use the source-dir-aware path so the imported module can in
        // turn locate its own imports.
        let analyzed = analyze_module_ast_with_source_dir(&ast, found_path.parent());
        // Walk every symbol's declared_type and qualify any internal
        // `Named { module: None, name: T }` references with `module:
        // Some(<this import name>)` when T is a top-level type symbol in
        // this imported module. Without this, an imported pointer alias
        // `Base = POINTER TO BaseDesc` appears as
        // `Pointer{target: Named{module: None, name: "BaseDesc"}}` to the
        // outer module, defeating cross-module record-extends checks.
        let local_type_names: HashSet<String> = analyzed
            .symbols
            .iter()
            .filter(|sym| sym.kind == SymbolKind::Type)
            .map(|sym| sym.name.clone())
            .collect();
        let qualified_symbols: Vec<SemanticSymbol> = analyzed
            .symbols
            .iter()
            .cloned()
            .map(|mut sym| {
                if let Some(ty) = sym.declared_type.as_mut() {
                    qualify_local_named_refs(ty, &import.name, &local_type_names);
                }
                sym
            })
            .collect();
        if debug {
            eprintln!(
                "[sema] import {} for module {} loaded from {} ({} symbols)",
                import.name,
                module.name,
                found_path.display(),
                qualified_symbols.len()
            );
        }
        out.insert(import.name.clone(), qualified_symbols);

        // Fold the imported module's own imported_modules table into
        // ours so cross-module type-alias chains that traverse more
        // than one hop resolve.  Each transitive entry's symbols were
        // already qualified by the inner analyser; we don't want to
        // re-qualify under the wrong module name, so merge as-is.
        // Direct entries take precedence on collision (an outer
        // import of X overrides any transitive sighting of X).
        for (transitive_name, transitive_syms) in analyzed.imported_modules {
            out.entry(transitive_name).or_insert(transitive_syms);
        }
    }
    out
}

/// Recursively rewrite `Named { module: None, name: T, .. }` references
/// inside `ty` to `Named { module: Some(module_name), name: T, kind:
/// Imported }` when `T` is in `local_type_names` (the set of top-level
/// type symbols defined by the imported module). This is the
/// post-processing step that lets cross-module record-extends checks see
/// the same canonical name on both sides of an inheritance edge.
pub fn qualify_local_named_refs(
    ty: &mut SemanticType,
    module_name: &str,
    local_type_names: &HashSet<String>,
) {
    match ty {
        SemanticType::Named { module: module_field, name, kind } => {
            if module_field.is_none() && local_type_names.contains(name) {
                *module_field = Some(module_name.to_string());
                *kind = NamedTypeKind::Imported;
            }
        }
        SemanticType::Array { element_type, .. } => {
            qualify_local_named_refs(element_type, module_name, local_type_names);
        }
        SemanticType::Record { base, fields, methods, .. } => {
            if let Some(base) = base.as_deref_mut() {
                qualify_local_named_refs(base, module_name, local_type_names);
            }
            for field in fields {
                qualify_local_named_refs(&mut field.ty, module_name, local_type_names);
            }
            for method in methods {
                qualify_proc_signature(&mut method.signature, module_name, local_type_names);
            }
        }
        SemanticType::Pointer { target, .. } => {
            qualify_local_named_refs(target, module_name, local_type_names);
        }
        SemanticType::Procedure(sig) => {
            qualify_proc_signature(sig, module_name, local_type_names);
        }
        _ => {}
    }
}

fn qualify_proc_signature(
    sig: &mut ProcedureType,
    module_name: &str,
    local_type_names: &HashSet<String>,
) {
    for param in &mut sig.parameters {
        qualify_local_named_refs(&mut param.ty, module_name, local_type_names);
    }
    if let Some(result) = sig.result_type.as_mut() {
        qualify_local_named_refs(result, module_name, local_type_names);
    }
}

fn is_simd_lane_count(lane_count: usize) -> bool {
    matches!(lane_count, 2 | 4 | 8 | 16)
}

fn builtin_type_names() -> HashSet<String> {
    builtin_types()
        .iter()
        .map(|builtin| builtin.name().to_string())
        .collect()
}

fn builtin_types() -> &'static [BuiltinType] {
    const BUILTINS: &[BuiltinType] = &[
        BuiltinType::AnyPtr,
        BuiltinType::AnyRec,
        BuiltinType::Boolean,
        BuiltinType::Byte,
        BuiltinType::Char,
        BuiltinType::IntShort,
        BuiltinType::Integer,
        BuiltinType::LongInt,
        BuiltinType::Real,
        BuiltinType::Set,
        BuiltinType::String,
        BuiltinType::ShortChar,
        BuiltinType::ShortInt,
        BuiltinType::ShortReal,
        BuiltinType::ShortString,
    ];
    BUILTINS
}

fn builtin_procs() -> &'static [BuiltinProc] {
    const BUILTINS: &[BuiltinProc] = &[
        BuiltinProc::Abs,
        BuiltinProc::Ash,
        BuiltinProc::Assert,
        BuiltinProc::Bits,
        BuiltinProc::Cap,
        BuiltinProc::Chr,
        BuiltinProc::Dec,
        BuiltinProc::Entier,
        BuiltinProc::Excl,
        BuiltinProc::Halt,
        BuiltinProc::Inc,
        BuiltinProc::Incl,
        BuiltinProc::Len,
        BuiltinProc::Long,
        BuiltinProc::Max,
        BuiltinProc::Min,
        BuiltinProc::New,
        BuiltinProc::Odd,
        BuiltinProc::Ord,
        BuiltinProc::Short,
        BuiltinProc::Size,
        BuiltinProc::SystemAdr,
        BuiltinProc::SystemVal,
        BuiltinProc::SystemLsh,
        BuiltinProc::SystemRot,
        BuiltinProc::SystemTyp,
        BuiltinProc::SystemBit,
        BuiltinProc::SystemGet,
        BuiltinProc::SystemPut,
        BuiltinProc::SystemMove,
        BuiltinProc::SystemNew,
        BuiltinProc::SystemGetReg,
        BuiltinProc::SystemPutReg,
    ];
    BUILTINS
}

fn builtin_type_by_name(name: &str) -> Option<BuiltinType> {
    match name {
        "ANYPTR" => Some(BuiltinType::AnyPtr),
        "ANYREC" => Some(BuiltinType::AnyRec),
        "BOOLEAN" => Some(BuiltinType::Boolean),
        "BYTE" => Some(BuiltinType::Byte),
        "CHAR" => Some(BuiltinType::Char),
        "INTSHORT" => Some(BuiltinType::IntShort),
        "INTEGER" => Some(BuiltinType::Integer),
        "LONGINT" => Some(BuiltinType::LongInt),
        "REAL" => Some(BuiltinType::Real),
        "SET" => Some(BuiltinType::Set),
        "String" => Some(BuiltinType::String),
        "SHORTCHAR" => Some(BuiltinType::ShortChar),
        "SHORTINT" => Some(BuiltinType::ShortInt),
        "SHORTREAL" => Some(BuiltinType::ShortReal),
        "Shortstring" => Some(BuiltinType::ShortString),
        _ => None,
    }
}

fn builtin_proc_by_name(name: &str) -> Option<BuiltinProc> {
    builtin_procs().iter().copied().find(|builtin| builtin.name() == name)
}

fn builtin_proc_result_type(
    proc: BuiltinProc,
    args: &[Expr],
    local_symbols: &[SemanticSymbol],
    outer_symbols: &[SemanticSymbol],
    scope_type_names: &HashSet<String>,
) -> Option<SemanticType> {
    match proc {
        BuiltinProc::Len | BuiltinProc::Ord | BuiltinProc::Size => {
            Some(SemanticType::Builtin(BuiltinType::Integer))
        }
        BuiltinProc::SystemAdr | BuiltinProc::SystemTyp => Some(SemanticType::Builtin(BuiltinType::Integer)),
        BuiltinProc::Odd => Some(SemanticType::Builtin(BuiltinType::Boolean)),
        BuiltinProc::Chr => Some(SemanticType::Builtin(BuiltinType::Char)),
        BuiltinProc::Cap => Some(SemanticType::Builtin(BuiltinType::Char)),
        BuiltinProc::Bits => Some(SemanticType::Builtin(BuiltinType::Set)),
        BuiltinProc::Entier => Some(SemanticType::Builtin(BuiltinType::LongInt)),
        BuiltinProc::Abs => args.first().and_then(|expr| {
            infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                .filter(is_numeric_type)
        }),
        BuiltinProc::Long => args.first().and_then(|expr| {
            infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                .and_then(long_result_type)
        }),
        BuiltinProc::Short => args.first().and_then(|expr| {
            infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                .and_then(short_result_type)
        }),
        BuiltinProc::Max | BuiltinProc::Min => {
            if args.len() == 1 {
                args.first().and_then(|expr| {
                    infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                        .map(|ty| match ty {
                            SemanticType::Builtin(BuiltinType::Set) => {
                                SemanticType::Builtin(BuiltinType::Integer)
                            }
                            other => other,
                        })
                })
            } else {
                let left = args.first().and_then(|expr| {
                    infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                });
                let right = args.get(1).and_then(|expr| {
                    infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                });
                match (left.as_ref(), right.as_ref()) {
                    (Some(left), Some(right)) if is_numeric_type(left) && is_numeric_type(right) => {
                        Some(promote_numeric_pair(left, right))
                    }
                    (Some(left), Some(right)) if is_character_like_type(left) && is_character_like_type(right) => {
                        Some(if character_rank(left) >= character_rank(right) {
                            left.clone()
                        } else {
                            right.clone()
                        })
                    }
                    _ => None,
                }
            }
        }
        BuiltinProc::Ash => args.first().and_then(|expr| {
            infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                .filter(is_integer_type)
        }),
        BuiltinProc::SystemVal => args.first().and_then(|expr| {
            infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
        }),
        BuiltinProc::SystemLsh | BuiltinProc::SystemRot => args.first().and_then(|expr| {
            infer_builtin_arg_type(expr, local_symbols, outer_symbols, scope_type_names)
                .filter(is_integer_type)
        }),
        BuiltinProc::New
        | BuiltinProc::Assert
        | BuiltinProc::Dec
        | BuiltinProc::Excl
        | BuiltinProc::Halt
        | BuiltinProc::Inc
        | BuiltinProc::Incl
        | BuiltinProc::SystemBit
        | BuiltinProc::SystemGet
        | BuiltinProc::SystemPut
        | BuiltinProc::SystemMove
        | BuiltinProc::SystemNew
        | BuiltinProc::SystemGetReg
        | BuiltinProc::SystemPutReg => None,
    }
}

fn infer_builtin_arg_type(
    expr: &Expr,
    local_symbols: &[SemanticSymbol],
    outer_symbols: &[SemanticSymbol],
    scope_type_names: &HashSet<String>,
) -> Option<SemanticType> {
    match expr {
        Expr::Literal { value, .. } => match value {
            newcp_parser::Literal::Integer(value) => Some(integer_literal_type(value)),
            newcp_parser::Literal::Real(_) => Some(SemanticType::Builtin(BuiltinType::Real)),
            newcp_parser::Literal::Character(value) => Some(character_literal_type(value)),
            newcp_parser::Literal::String(value) => Some(string_literal_type(value)),
        },
        Expr::Nil { .. } => Some(SemanticType::Nil),
        Expr::Designator(designator) if designator.base.module.is_none() && designator.selectors.is_empty() => {
            if let Some(builtin) = builtin_type_by_name(&designator.base.name) {
                return Some(SemanticType::Builtin(builtin));
            }
            local_symbols
                .iter()
                .find(|symbol| symbol.name == designator.base.name)
                .or_else(|| outer_symbols.iter().find(|symbol| symbol.name == designator.base.name))
                .and_then(|symbol| symbol.declared_type.clone())
        }
        Expr::Designator(designator) if designator.base.module.is_none() && designator.selectors.is_empty() => {
            if scope_type_names.contains(&designator.base.name) {
                Some(SemanticType::Named {
                    module: None,
                    name: designator.base.name.clone(),
                    kind: NamedTypeKind::UserDefined,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn long_result_type(ty: SemanticType) -> Option<SemanticType> {
    match ty {
        SemanticType::Builtin(BuiltinType::Byte) => Some(SemanticType::Builtin(BuiltinType::ShortInt)),
        SemanticType::Builtin(BuiltinType::ShortInt) => Some(SemanticType::Builtin(BuiltinType::IntShort)),
        SemanticType::Builtin(BuiltinType::IntShort) => Some(SemanticType::Builtin(BuiltinType::Integer)),
        SemanticType::Builtin(BuiltinType::Integer) => Some(SemanticType::Builtin(BuiltinType::LongInt)),
        SemanticType::Builtin(BuiltinType::ShortReal) => Some(SemanticType::Builtin(BuiltinType::Real)),
        SemanticType::Builtin(BuiltinType::ShortChar) => Some(SemanticType::Builtin(BuiltinType::Char)),
        SemanticType::Builtin(BuiltinType::ShortString) => Some(SemanticType::Builtin(BuiltinType::String)),
        _ => None,
    }
}

fn short_result_type(ty: SemanticType) -> Option<SemanticType> {
    match ty {
        SemanticType::Builtin(BuiltinType::LongInt) => Some(SemanticType::Builtin(BuiltinType::Integer)),
        SemanticType::Builtin(BuiltinType::Integer) => Some(SemanticType::Builtin(BuiltinType::IntShort)),
        SemanticType::Builtin(BuiltinType::IntShort) => Some(SemanticType::Builtin(BuiltinType::ShortInt)),
        SemanticType::Builtin(BuiltinType::ShortInt) => Some(SemanticType::Builtin(BuiltinType::Byte)),
        SemanticType::Builtin(BuiltinType::Real) => Some(SemanticType::Builtin(BuiltinType::ShortReal)),
        SemanticType::Builtin(BuiltinType::Char) => Some(SemanticType::Builtin(BuiltinType::ShortChar)),
        SemanticType::Builtin(BuiltinType::String) => Some(SemanticType::Builtin(BuiltinType::ShortString)),
        _ => None,
    }
}

fn selector_resolution_kind_for_type(ty: &SemanticType) -> SelectorResolutionKind {
    match ty {
        SemanticType::Builtin(_) => SelectorResolutionKind::TypeGuard,
        SemanticType::Named {
            kind: NamedTypeKind::UserDefined,
            ..
        }
        | SemanticType::Named {
            kind: NamedTypeKind::Imported,
            ..
        } => SelectorResolutionKind::TypeGuard,
        SemanticType::Named {
            kind: NamedTypeKind::Unresolved,
            ..
        } => SelectorResolutionKind::Unresolved,
        _ => SelectorResolutionKind::TypeGuard,
    }
}

fn infer_additive_or_multiplicative_result(
    op: BinaryOp,
    left: Option<&SemanticType>,
    right: Option<&SemanticType>,
) -> Option<SemanticType> {
    match (left, right) {
        (Some(left), Some(right)) if is_numeric_type(left) && is_numeric_type(right) => {
            Some(promote_numeric_pair(left, right))
        }
        (Some(SemanticType::Builtin(BuiltinType::Set)), Some(SemanticType::Builtin(BuiltinType::Set)))
            if matches!(op, BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply) =>
        {
            Some(SemanticType::Builtin(BuiltinType::Set))
        }
        (Some(left), Some(right))
            if op == BinaryOp::Add && is_string_type(left) && is_string_type(right) =>
        {
            Some(if matches!(left, SemanticType::Builtin(BuiltinType::ShortString))
                && matches!(right, SemanticType::Builtin(BuiltinType::ShortString))
            {
                SemanticType::Builtin(BuiltinType::ShortString)
            } else {
                SemanticType::Builtin(BuiltinType::String)
            })
        }
        _ => None,
    }
}

fn are_relation_compatible(left: &SemanticType, right: &SemanticType) -> bool {
    // NIL is assignment-compatible with any pointer or procedure type,
    // including named types that alias pointers/procedures, plus
    // ANYPTR (the universal pointer supertype).
    let nil_compatible = |other: &SemanticType| {
        matches!(
            other,
            SemanticType::Pointer { .. }
                | SemanticType::Procedure(_)
                | SemanticType::Named { .. }
                | SemanticType::Builtin(BuiltinType::AnyPtr)
        )
    };
    // ANYPTR compares with any pointer / procedure / ANYPTR — for `=`
    // / `#`, identity comparison is well-defined across the universal
    // supertype.
    let anyptr_compatible = |other: &SemanticType| {
        matches!(
            other,
            SemanticType::Pointer { .. }
                | SemanticType::Procedure(_)
                | SemanticType::Named { .. }
                | SemanticType::Builtin(BuiltinType::AnyPtr)
                | SemanticType::Nil
        )
    };
    left == right
        || (is_numeric_type(left) && is_numeric_type(right))
        || (is_character_like_type(left) && is_character_like_type(right))
        || (is_string_like_type(left) && is_string_like_type(right))
        || (matches!(left, SemanticType::Nil) && nil_compatible(right))
        || (matches!(right, SemanticType::Nil) && nil_compatible(left))
        || (matches!(left, SemanticType::Builtin(BuiltinType::AnyPtr)) && anyptr_compatible(right))
        || (matches!(right, SemanticType::Builtin(BuiltinType::AnyPtr)) && anyptr_compatible(left))
        || are_pointer_types_relation_compatible(left, right)
}

/// Pointer-to-record relation compatibility for `=` / `#`.  CP allows
/// comparing pointers whose target record types lie on the same
/// inheritance chain (one extends the other, possibly via several
/// ancestors).  Used so a base-typed handle can be `=`-compared
/// against a subtype handle without sema rejecting it.
fn are_pointer_types_relation_compatible(left: &SemanticType, right: &SemanticType) -> bool {
    let (SemanticType::Pointer { target: lt, .. }, SemanticType::Pointer { target: rt, .. }) =
        (left, right)
    else {
        return false;
    };
    // Both pointer targets must be (named) record types. We don't have
    // the analyser's import map at this layer, so we accept any pair
    // where the targets are syntactically related: identical, or one
    // is a Named ref whose base — locally observable via the Record's
    // `base` chain — eventually matches the other. The full extension
    // walk happens later in IR; this gate just stops the "obviously
    // unrelated pointers" diagnostic.
    record_chains_overlap(lt, rt)
}

fn record_chains_overlap(a: &SemanticType, b: &SemanticType) -> bool {
    // Either side must be a Record (or a Named ref to a record). If
    // we only see Named-without-resolved-base on one side, accept —
    // we can't disprove the relation at this layer.
    if a == b {
        return true;
    }
    if let (SemanticType::Named { module: am, name: an, .. },
            SemanticType::Named { module: bm, name: bn, .. }) = (a, b) {
        if am == bm && an == bn {
            return true;
        }
    }
    // Walk a's base chain looking for b.
    let mut cur = a;
    loop {
        match cur {
            SemanticType::Record { base: Some(parent), .. } => {
                if parent.as_ref() == b {
                    return true;
                }
                cur = parent.as_ref();
            }
            _ => break,
        }
    }
    // Walk b's base chain looking for a.
    let mut cur = b;
    loop {
        match cur {
            SemanticType::Record { base: Some(parent), .. } => {
                if parent.as_ref() == a {
                    return true;
                }
                cur = parent.as_ref();
            }
            _ => break,
        }
    }
    // Conservatively accept when at least one side is a Named ref we
    // can't fully inspect at this layer — false negatives here are
    // worse than false positives because the IR layer re-validates.
    matches!(a, SemanticType::Named { .. }) || matches!(b, SemanticType::Named { .. })
}

/// Like is_string_type but also accepts ARRAY OF CHAR / ARRAY OF
/// SHORTCHAR — CP treats `name # "lit"` as valid when name is a
/// fixed CHAR array and the literal is a string. Used by relation
/// compatibility so `name # "expected"` doesn't trip on the
/// declared-array-vs-string-literal mismatch.
fn is_string_like_type(ty: &SemanticType) -> bool {
    if is_string_type(ty) {
        return true;
    }
    if is_character_like_type(ty) {
        return true;
    }
    matches!(
        ty,
        SemanticType::Array { element_type, .. }
            if matches!(
                element_type.as_ref(),
                SemanticType::Builtin(BuiltinType::Char)
                    | SemanticType::Builtin(BuiltinType::ShortChar)
            )
    )
}

fn are_ordered_relation_compatible(left: &SemanticType, right: &SemanticType) -> bool {
    // CP §8.2.5: `<` / `<=` / `>` / `>=` apply to numerics, CHAR/SHORTCHAR,
    // string types (lexicographic — covers both `String` builtins and
    // CHAR-arrays), and SETs (subset / superset).
    (is_numeric_type(left) && is_numeric_type(right))
        || (is_character_like_type(left) && is_character_like_type(right))
        || (is_string_like_type(left) && is_string_like_type(right))
        || matches!(
            (left, right),
            (
                SemanticType::Builtin(BuiltinType::Set),
                SemanticType::Builtin(BuiltinType::Set),
            )
        )
}


fn is_boolean_type(ty: &SemanticType) -> bool {
    matches!(ty, SemanticType::Builtin(BuiltinType::Boolean))
}

fn is_character_like_type(ty: &SemanticType) -> bool {
    matches!(
        ty,
        SemanticType::Builtin(BuiltinType::Char) | SemanticType::Builtin(BuiltinType::ShortChar)
    )
}

fn is_integer_type(ty: &SemanticType) -> bool {
    matches!(
        ty,
        SemanticType::Builtin(BuiltinType::Byte)
            | SemanticType::Builtin(BuiltinType::ShortInt)
            | SemanticType::Builtin(BuiltinType::IntShort)
            | SemanticType::Builtin(BuiltinType::Integer)
            | SemanticType::Builtin(BuiltinType::LongInt)
    )
}

fn is_numeric_type(ty: &SemanticType) -> bool {
    is_integer_type(ty)
        || matches!(
            ty,
            SemanticType::Builtin(BuiltinType::ShortReal)
                | SemanticType::Builtin(BuiltinType::Real)
        )
}

fn is_string_type(ty: &SemanticType) -> bool {
    matches!(
        ty,
        SemanticType::Builtin(BuiltinType::ShortString) | SemanticType::Builtin(BuiltinType::String)
    )
}

fn integer_literal_type(value: &str) -> SemanticType {
    if value.ends_with('L') {
        SemanticType::Builtin(BuiltinType::LongInt)
    } else if value.ends_with('H') {
        SemanticType::Builtin(BuiltinType::IntShort)
    } else {
        SemanticType::Builtin(BuiltinType::Integer)
    }
}

fn character_literal_type(value: &str) -> SemanticType {
    let digits = value.strip_suffix('X').unwrap_or(value);
    let ordinal = i64::from_str_radix(digits, 16).ok().unwrap_or(0);
    if ordinal <= 0xFF {
        SemanticType::Builtin(BuiltinType::ShortChar)
    } else {
        SemanticType::Builtin(BuiltinType::Char)
    }
}

fn string_literal_type(value: &str) -> SemanticType {
    let inner = value.trim_matches(['\'', '"']);
    if inner.chars().count() == 1 {
        return SemanticType::Builtin(BuiltinType::Char);
    }
    let contains_non_latin1 = inner.chars().any(|ch| (ch as u32) > 0xFF);
    if contains_non_latin1 {
        SemanticType::Builtin(BuiltinType::String)
    } else {
        SemanticType::Builtin(BuiltinType::ShortString)
    }
}

fn promote_integer_pair(left: &SemanticType, right: &SemanticType) -> SemanticType {
    if numeric_rank(left) >= numeric_rank(right) {
        left.clone()
    } else {
        right.clone()
    }
}

fn promote_numeric_pair(left: &SemanticType, right: &SemanticType) -> SemanticType {
    if numeric_rank(left) >= numeric_rank(right) {
        left.clone()
    } else {
        right.clone()
    }
}

fn numeric_rank(ty: &SemanticType) -> usize {
    match ty {
        SemanticType::Builtin(BuiltinType::Byte) => 0,
        SemanticType::Builtin(BuiltinType::ShortInt) => 1,
        SemanticType::Builtin(BuiltinType::IntShort) => 2,
        SemanticType::Builtin(BuiltinType::Integer) => 3,
        SemanticType::Builtin(BuiltinType::LongInt) => 4,
        SemanticType::Builtin(BuiltinType::ShortReal) => 5,
        SemanticType::Builtin(BuiltinType::Real) => 6,
        _ => usize::MAX,
    }
}

fn character_rank(ty: &SemanticType) -> usize {
    match ty {
        SemanticType::Builtin(BuiltinType::ShortChar) => 0,
        SemanticType::Builtin(BuiltinType::Char) => 1,
        _ => usize::MAX,
    }
}

fn is_case_selector_type(ty: &SemanticType) -> bool {
    is_integer_type(ty) || is_character_like_type(ty)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseValueKind {
    Integer,
    Character,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvaluatedCaseValue {
    kind: CaseValueKind,
    value: i128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvaluatedCaseRange {
    kind: CaseValueKind,
    start: i128,
    end: i128,
}

/// Evaluate any compile-time constant expression into a `ConstValue`.
/// Returns `None` only for expressions that genuinely cannot be evaluated
/// at compile time (e.g. function calls, non-const designators).
fn evaluate_const_expr(
    expr: &Expr,
    local_symbols: &[SemanticSymbol],
    module_symbols: &[SemanticSymbol],
) -> Option<ConstValue> {
    evaluate_const_expr_with_imports(expr, local_symbols, module_symbols, None)
}

fn evaluate_const_expr_with_imports(
    expr: &Expr,
    local_symbols: &[SemanticSymbol],
    module_symbols: &[SemanticSymbol],
    imports: Option<&HashMap<String, Vec<SemanticSymbol>>>,
) -> Option<ConstValue> {
    let recur = |e: &Expr| {
        evaluate_const_expr_with_imports(e, local_symbols, module_symbols, imports)
    };
    match expr {
        Expr::Literal { value, .. } => match value {
            newcp_parser::Literal::Integer(s) => {
                parse_component_pascal_integer(s).map(ConstValue::Integer)
            }
            newcp_parser::Literal::Real(s) => {
                s.parse::<f64>().ok().map(ConstValue::Real)
            }
            newcp_parser::Literal::String(s) => {
                // A single-character string literal is a CHAR constant.
                // The lexer stores the raw delimited form; strip the quotes.
                let inner = strip_string_delimiters(s);
                if inner.chars().count() == 1 {
                    Some(ConstValue::Char(inner.chars().next().unwrap()))
                } else {
                    Some(ConstValue::String(inner.to_string()))
                }
            }
            newcp_parser::Literal::Character(s) => {
                // Character literals are either 'X' or NNX (hex char code).
                parse_component_pascal_character(s)
                    .and_then(|code| char::from_u32(code as u32))
                    .map(ConstValue::Char)
            }
        },
        Expr::Nil { .. } => None,
        // Module-qualified constant — `Kernel.timeResolution`,
        // `Sequencers.clean`, etc.  Look up in the imported
        // module's symbol table when we have one threaded
        // through; otherwise fall through to None.  Without
        // this branch, every derived const that mentions an
        // imported const folded to None and the receiver
        // dropped out of the local CONST symbol table —
        // surfacing later as "identifier <derived> is not
        // declared" at every use site.
        Expr::Designator(d) if d.selectors.is_empty() && d.base.module.is_some() => {
            let module_name = d.base.module.as_deref()?;
            let module_syms = imports?.get(module_name)?;
            module_syms.iter()
                .find(|s| s.name == d.base.name && s.kind == SymbolKind::Constant)
                .and_then(|s| s.const_value.clone())
        }
        Expr::Designator(d) if d.selectors.is_empty() && d.base.module.is_none() => {
            // TRUE / FALSE are keyword-like but stored as builtin constants.
            match d.base.name.as_str() {
                "TRUE"  => return Some(ConstValue::Boolean(true)),
                "FALSE" => return Some(ConstValue::Boolean(false)),
                _ => {}
            }
            local_symbols.iter().rev()
                .chain(module_symbols.iter().rev())
                .find(|s| s.name == d.base.name && s.kind == SymbolKind::Constant)
                .and_then(|s| s.const_value.clone())
        }
        Expr::Unary { op, expr, .. } => {
            let inner = recur(expr)?;
            match (op, inner) {
                (newcp_parser::UnaryOp::Plus,  ConstValue::Integer(v)) => Some(ConstValue::Integer(v)),
                (newcp_parser::UnaryOp::Minus, ConstValue::Integer(v)) => v.checked_neg().map(ConstValue::Integer),
                (newcp_parser::UnaryOp::Plus,  ConstValue::Real(v))    => Some(ConstValue::Real(v)),
                (newcp_parser::UnaryOp::Minus, ConstValue::Real(v))    => Some(ConstValue::Real(-v)),
                (newcp_parser::UnaryOp::Not,   ConstValue::Boolean(v)) => Some(ConstValue::Boolean(!v)),
                _ => None,
            }
        }
        Expr::Set { elements, .. } => {
            let mut bits: u32 = 0;
            for elem in elements {
                let start = match recur(&elem.start)? {
                    ConstValue::Integer(n) if (0..32).contains(&n) => n as u32,
                    _ => return None,
                };
                let end = if let Some(end_expr) = &elem.end {
                    match recur(end_expr)? {
                        ConstValue::Integer(n) if (0..32).contains(&n) => n as u32,
                        _ => return None,
                    }
                } else {
                    start
                };
                if end < start {
                    return None;
                }
                for b in start..=end {
                    bits |= 1u32 << b;
                }
            }
            Some(ConstValue::Set(bits))
        }
        Expr::Binary { left, op, right, .. } => {
            let lv = recur(left)?;
            let rv = recur(right)?;
            match (lv, rv) {
                (ConstValue::Integer(l), ConstValue::Integer(r)) => {
                    match op {
                        BinaryOp::Add      => l.checked_add(r).map(ConstValue::Integer),
                        BinaryOp::Subtract => l.checked_sub(r).map(ConstValue::Integer),
                        BinaryOp::Multiply => l.checked_mul(r).map(ConstValue::Integer),
                        BinaryOp::Div      => if r == 0 { None } else { l.checked_div(r).map(ConstValue::Integer) },
                        BinaryOp::Mod      => if r == 0 { None } else { l.checked_rem(r).map(ConstValue::Integer) },
                        BinaryOp::Equal    => Some(ConstValue::Boolean(l == r)),
                        BinaryOp::NotEqual => Some(ConstValue::Boolean(l != r)),
                        BinaryOp::Less     => Some(ConstValue::Boolean(l <  r)),
                        BinaryOp::LessEqual    => Some(ConstValue::Boolean(l <= r)),
                        BinaryOp::Greater      => Some(ConstValue::Boolean(l >  r)),
                        BinaryOp::GreaterEqual => Some(ConstValue::Boolean(l >= r)),
                        _ => None,
                    }
                }
                (ConstValue::Real(l), ConstValue::Real(r)) => {
                    match op {
                        BinaryOp::Add      => Some(ConstValue::Real(l + r)),
                        BinaryOp::Subtract => Some(ConstValue::Real(l - r)),
                        BinaryOp::Multiply => Some(ConstValue::Real(l * r)),
                        BinaryOp::Divide   => Some(ConstValue::Real(l / r)),
                        _ => None,
                    }
                }
                (ConstValue::Boolean(l), ConstValue::Boolean(r)) => {
                    match op {
                        BinaryOp::And      => Some(ConstValue::Boolean(l && r)),
                        BinaryOp::Or       => Some(ConstValue::Boolean(l || r)),
                        BinaryOp::Equal    => Some(ConstValue::Boolean(l == r)),
                        BinaryOp::NotEqual => Some(ConstValue::Boolean(l != r)),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Strip surrounding quote characters from a string literal as stored by the lexer.
/// Handles both `"..."` and `'...'` forms.
fn strip_string_delimiters(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn evaluate_const_integer(
    expr: &Expr,
    local_symbols: &[SemanticSymbol],
    module_symbols: &[SemanticSymbol],
) -> Option<ConstValue> {
    evaluate_const_expr(expr, local_symbols, module_symbols)
        .filter(|v| matches!(v, ConstValue::Integer(_)))
}

/// Infer the CP builtin type that corresponds to a constant value.
fn const_value_type(value: &Option<ConstValue>) -> Option<SemanticType> {
    match value {
        Some(ConstValue::Integer(_)) => Some(SemanticType::Builtin(BuiltinType::Integer)),
        Some(ConstValue::Real(_))    => Some(SemanticType::Builtin(BuiltinType::Real)),
        Some(ConstValue::Char(_))    => Some(SemanticType::Builtin(BuiltinType::Char)),
        Some(ConstValue::String(_))  => Some(SemanticType::Builtin(BuiltinType::String)),
        Some(ConstValue::Boolean(_)) => Some(SemanticType::Builtin(BuiltinType::Boolean)),
        Some(ConstValue::Set(_))     => Some(SemanticType::Builtin(BuiltinType::Set)),
        None => None,
    }
}

fn evaluate_case_label_value(
    expr: &Expr,
    _local_symbols: &[SemanticSymbol],
) -> Option<EvaluatedCaseValue> {
    match expr {
        Expr::Literal { value, .. } => match value {
            newcp_parser::Literal::Integer(value) => parse_component_pascal_integer(value).map(|value| {
                EvaluatedCaseValue {
                    kind: CaseValueKind::Integer,
                    value,
                }
            }),
            newcp_parser::Literal::Character(value) => parse_component_pascal_character(value).map(|value| {
                EvaluatedCaseValue {
                    kind: CaseValueKind::Character,
                    value,
                }
            }),
            newcp_parser::Literal::Real(_) | newcp_parser::Literal::String(_) => None,
        },
        Expr::Unary { op, expr, .. } => {
            let inner = evaluate_case_label_value(expr, _local_symbols)?;
            match op {
                newcp_parser::UnaryOp::Plus if inner.kind == CaseValueKind::Integer => Some(inner),
                newcp_parser::UnaryOp::Minus if inner.kind == CaseValueKind::Integer => {
                    Some(EvaluatedCaseValue {
                        kind: inner.kind,
                        value: inner.value.checked_neg()?,
                    })
                }
                _ => None,
            }
        }
        Expr::Binary { left, op, right, .. } => {
            let left = evaluate_case_label_value(left, _local_symbols)?;
            let right = evaluate_case_label_value(right, _local_symbols)?;
            if left.kind != CaseValueKind::Integer || right.kind != CaseValueKind::Integer {
                return None;
            }

            let value = match op {
                BinaryOp::Add => left.value.checked_add(right.value)?,
                BinaryOp::Subtract => left.value.checked_sub(right.value)?,
                BinaryOp::Multiply => left.value.checked_mul(right.value)?,
                BinaryOp::Div => {
                    if right.value == 0 {
                        return None;
                    }
                    left.value.checked_div(right.value)?
                }
                BinaryOp::Mod => {
                    if right.value == 0 {
                        return None;
                    }
                    left.value.checked_rem(right.value)?
                }
                _ => return None,
            };

            Some(EvaluatedCaseValue {
                kind: CaseValueKind::Integer,
                value,
            })
        }
        Expr::Designator(_) | Expr::Nil { .. } | Expr::Set { .. } => None,
    }
}

fn parse_component_pascal_integer(text: &str) -> Option<i128> {
    if let Some(value) = text.strip_suffix('H') {
        i128::from_str_radix(value, 16).ok()
    } else if let Some(value) = text.strip_suffix('L') {
        i128::from_str_radix(value, 16).ok()
    } else {
        text.parse::<i128>().ok()
    }
}

fn parse_component_pascal_character(text: &str) -> Option<i128> {
    text.strip_suffix('X')
        .and_then(|value| i128::from_str_radix(value, 16).ok())
}

/// Extract the compile-time integer value of `expr`, if it is a literal
/// integer (optionally wrapped in unary `+`/`-`). Returns `None` for
/// any expression that requires runtime evaluation.
///
/// This intentionally does not flow constants through binary expressions
/// or named CONSTs — the use case is purely "is this an integer literal
/// that should adapt to the target type", which is the polymorphic-
/// integer-literal rule in CP.
fn extract_integer_literal_value(expr: &Expr) -> Option<i128> {
    match expr {
        Expr::Literal { value: Literal::Integer(text), .. } => {
            parse_component_pascal_integer(text)
        }
        Expr::Literal { value: Literal::Character(text), .. } => {
            parse_component_pascal_character(text)
        }
        Expr::Unary { op: UnaryOp::Minus, expr: inner, .. } => {
            extract_integer_literal_value(inner).map(|n| -n)
        }
        Expr::Unary { op: UnaryOp::Plus, expr: inner, .. } => {
            extract_integer_literal_value(inner)
        }
        _ => None,
    }
}

/// Inclusive `(min, max)` value range for each integer builtin type, in
/// the i128 domain. Used to check whether a constant literal fits the
/// target type. Returns `None` for non-integer types (including CHAR
/// and the SET/REAL/etc. families).
fn integer_type_range(ty: &SemanticType) -> Option<(i128, i128)> {
    match ty {
        SemanticType::Builtin(BuiltinType::Byte) => Some((0, 255)),
        SemanticType::Builtin(BuiltinType::ShortInt) => Some((i16::MIN as i128, i16::MAX as i128)),
        SemanticType::Builtin(BuiltinType::IntShort) => Some((i32::MIN as i128, i32::MAX as i128)),
        SemanticType::Builtin(BuiltinType::Integer)
        | SemanticType::Builtin(BuiltinType::LongInt) => Some((i64::MIN as i128, i64::MAX as i128)),
        _ => None,
    }
}

/// True if `expr` is a constant integer literal whose value fits in
/// the integer-typed `target`. Implements the CP rule that integer
/// literals are polymorphic with respect to the assignment / argument-
/// passing context.
fn integer_literal_fits_target(expr: &Expr, target: &SemanticType) -> bool {
    let Some(value) = extract_integer_literal_value(expr) else { return false };
    let Some((lo, hi)) = integer_type_range(target) else { return false };
    value >= lo && value <= hi
}

fn render_case_value_range(start: i128, end: i128, kind: CaseValueKind) -> String {
    if start == end {
        render_case_value(start, kind)
    } else {
        format!(
            "{}..{}",
            render_case_value(start, kind),
            render_case_value(end, kind)
        )
    }
}

fn render_case_value(value: i128, kind: CaseValueKind) -> String {
    match kind {
        CaseValueKind::Integer => value.to_string(),
        CaseValueKind::Character => format!("{:X}X", value),
    }
}

fn make_diagnostic(
    procedure: Option<&str>,
    line: usize,
    column: usize,
    message: String,
) -> SemanticDiagnostic {
    SemanticDiagnostic {
        procedure: procedure.map(str::to_string),
        line,
        column,
        message,
    }
}

fn statement_position(statement: &Statement) -> (usize, usize) {
    match statement {
        Statement::Empty { span }
        | Statement::Assignment { span, .. }
        | Statement::ProcedureCall { span, .. }
        | Statement::If { span, .. }
        | Statement::Case { span, .. }
        | Statement::While { span, .. }
        | Statement::Repeat { span, .. }
        | Statement::For { span, .. }
        | Statement::Loop { span, .. }
        | Statement::With { span, .. }
        | Statement::Exit { span }
        | Statement::Brk { span, .. }
        | Statement::Return { span, .. } => (span.start.line, span.start.column),
    }
}

fn expr_position(expr: &Expr) -> (usize, usize) {
    match expr {
        Expr::Literal { span, .. }
        | Expr::Nil { span }
        | Expr::Set { span, .. }
        | Expr::Unary { span, .. }
        | Expr::Binary { span, .. } => (span.start.line, span.start.column),
        Expr::Designator(designator) => (designator.span.start.line, designator.span.start.column),
    }
}

fn designator_position(designator: &Designator) -> (usize, usize) {
    (designator.span.start.line, designator.span.start.column)
}

fn render_symbol_row(sym: &SemanticSymbol) -> String {
    let export_mark = if sym.read_only_export {
        "-"
    } else if sym.exported {
        "*"
    } else {
        " "
    };
    let kind = match sym.kind {
        SymbolKind::Constant  => "const",
        SymbolKind::Type      => "type ",
        SymbolKind::Variable  => "var  ",
        SymbolKind::Procedure => "proc ",
        SymbolKind::Import    => "import",
        SymbolKind::Parameter => "param",
        SymbolKind::Receiver  => "recv ",
    };
    let type_str = sym
        .declared_type
        .as_ref()
        .map(|ty| format!(" : {}", render_semantic_type(ty)))
        .unwrap_or_default();
    let val_str = sym
        .const_value
        .as_ref()
        .map(|v| match v {
            ConstValue::Integer(n) => format!(" = {n}"),
            ConstValue::Real(f)    => format!(" = {f}"),
            ConstValue::Char(c)    => format!(" = '{c}'"),
            ConstValue::String(s)  => format!(" = \"{s}\""),
            ConstValue::Boolean(b) => format!(" = {b}"),
            ConstValue::Set(bits)  => format!(" = {{ bits=0x{bits:08x} }}"),
        })
        .unwrap_or_default();
    let simd_str = sym
        .simd_shape
        .as_ref()
        .map(|s| format!(" [simd:{}]", render_simd_shape(s)))
        .unwrap_or_default();
    format!("  {kind} {export_mark}{}{type_str}{val_str}{simd_str}", sym.name)
}

fn render_resolution(r: &SelectorResolution) -> String {
    format!(
        "    {} `{}` -> [{}]{}",
        r.designator,
        r.selector,
        r.kind.as_str(),
        if r.reason.is_empty() { String::new() } else { format!(" // {}", r.reason) }
    )
}

fn render_diagnostic_item(d: &SemanticDiagnostic) -> String {
    format!("    ERROR {}:{} {}", d.line, d.column, d.message)
}

fn render_module_dump(path: &Path, module: &SemanticModule) -> String {
    let mut out = Vec::<String>::new();

    out.push(format!("=== sema dump: {} ===", path.display()));
    out.push(format!("module: {}", module.name));

    // imports
    if module.imports.is_empty() {
        out.push("imports: <none>".to_string());
    } else {
        out.push(format!("imports: {}", module.imports.join(", ")));
    }

    out.push(String::new());

    // module-level symbols
    out.push("--- module symbols ---".to_string());
    if module.symbols.is_empty() {
        out.push("  <none>".to_string());
    } else {
        for sym in &module.symbols {
            out.push(render_symbol_row(sym));
        }
    }

    // module-level selector resolutions (body of module BEGIN block)
    if !module.selector_resolutions.is_empty() {
        out.push(String::new());
        out.push("--- module-level selector resolutions ---".to_string());
        for r in &module.selector_resolutions {
            out.push(render_resolution(r));
        }
    }

    // module-level diagnostics
    if !module.diagnostics.is_empty() {
        out.push(String::new());
        out.push("--- module-level diagnostics ---".to_string());
        for d in &module.diagnostics {
            out.push(render_diagnostic_item(d));
        }
    }

    // per-procedure detail
    for proc in &module.procedures {
        out.push(String::new());
        let export_mark = if proc.exported { "*" } else { "" };
        out.push(format!(
            "--- procedure {}{} ---",
            export_mark,
            proc.name
        ));

        // signature
        let params: Vec<String> = proc.signature.parameters.iter().map(|p| {
            let mode = match p.mode {
                Some(ParamMode::Var) => "VAR ",
                Some(ParamMode::In)  => "IN ",
                Some(ParamMode::Out) => "OUT ",
                None => "",
            };
            let names = p.names.join(", ");
            format!("{}{}: {}", mode, names, render_semantic_type(&p.ty))
        }).collect();
        let ret = proc.signature.result_type.as_ref()
            .map(|t| format!(" : {}", render_semantic_type(t)))
            .unwrap_or_default();
        out.push(format!("  signature: ({}){}", params.join("; "), ret));

        // local symbols
        if proc.local_symbols.is_empty() {
            out.push("  locals: <none>".to_string());
        } else {
            out.push("  locals:".to_string());
            for sym in &proc.local_symbols {
                out.push(render_symbol_row(sym));
            }
        }

        // selector resolutions
        if proc.selector_resolutions.is_empty() {
            out.push("  resolutions: <none>".to_string());
        } else {
            out.push("  resolutions:".to_string());
            for r in &proc.selector_resolutions {
                out.push(render_resolution(r));
            }
        }

        // diagnostics
        if proc.diagnostics.is_empty() {
            out.push("  diagnostics: <none>".to_string());
        } else {
            out.push("  diagnostics:".to_string());
            for d in &proc.diagnostics {
                out.push(render_diagnostic_item(d));
            }
        }
    }

    out.push(String::new());
    out.push(format!(
        "=== end: {} symbols, {} procedures ===",
        module.symbols.len(),
        module.procedures.len()
    ));

    out.join("\n")
}

fn render_simd_shape(shape: &SimdShape) -> String {
    format!(
        "{}:{}x{}:{}B",
        shape.layout.as_str(),
        shape.lane_kind.as_str(),
        shape.lane_count,
        shape.packed_bytes
    )
}

fn render_semantic_type(ty: &SemanticType) -> String {
    match ty {
        SemanticType::Builtin(builtin) => builtin.name().to_string(),
        SemanticType::BuiltinProc(builtin) => format!("builtin:{}", builtin.name()),
        SemanticType::Nil => "NIL".to_string(),
        SemanticType::Named { module, name, kind } => match kind {
            NamedTypeKind::UserDefined => format!("type:{}", render_optional_module_name(module, name)),
            NamedTypeKind::Imported => format!("imported:{}", render_optional_module_name(module, name)),
            NamedTypeKind::Unresolved => format!("unresolved:{}", render_optional_module_name(module, name)),
        },
        SemanticType::Array {
            lengths,
            element_type,
            untagged,
        } => format!(
            "ARRAY{} {} OF {}",
            if *untagged { " [untagged]" } else { "" },
            lengths.join(", "),
            render_semantic_type(element_type)
        ),
        SemanticType::Record { flavor, layout, base, fields, methods } => {
            let flavor = flavor
                .map(|item| match item {
                    RecordFlavor::Abstract => "ABSTRACT ",
                    RecordFlavor::Extensible => "EXTENSIBLE ",
                    RecordFlavor::Limited => "LIMITED ",
                })
                .unwrap_or("");
            let layout = render_record_layout(*layout);
            let base = base
                .as_ref()
                .map(|item| format!("({}) ", render_semantic_type(item)))
                .unwrap_or_default();
            let fields = if fields.is_empty() && methods.is_empty() {
                String::new()
            } else {
                let mut items = fields
                    .iter()
                    .map(|field| format!("{}: {}", field.names.join(", "), render_semantic_type(&field.ty)))
                    .collect::<Vec<_>>();
                items.extend(
                    methods
                        .iter()
                        .map(|method| format!("{}: {}", method.name, render_procedure_type(&method.signature))),
                );
                format!(
                    " {}",
                    items.join("; ")
                )
            };
            format!("{}RECORD {}{}{} END", flavor, layout, base, fields)
        }
        SemanticType::Pointer { target, untagged } => format!(
            "POINTER{} TO {}",
            if *untagged { " [untagged]" } else { "" },
            render_semantic_type(target)
        ),
        SemanticType::Procedure(signature) => render_procedure_type(signature),
    }
}

fn render_record_layout(layout: RecordLayout) -> &'static str {
    match layout {
        RecordLayout::Tagged => "",
        RecordLayout::Untagged => "[untagged] ",
        RecordLayout::UntaggedNoAlign => "[noalign] ",
        RecordLayout::UntaggedAlign2 => "[align2] ",
        RecordLayout::UntaggedAlign8 => "[align8] ",
        RecordLayout::Union => "[union] ",
    }
}

fn render_sys_flag(flag: &SysFlag) -> String {
    match flag {
        SysFlag::Named(name) => format!("[{name}]"),
        SysFlag::Numeric(value) => format!("[{value}]"),
    }
}

fn render_procedure_type(signature: &ProcedureType) -> String {
    let receiver = signature
        .receiver
        .as_ref()
        .map(|receiver| format!("({}) ", render_semantic_type(receiver)))
        .unwrap_or_default();
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            let mode = match parameter.mode {
                Some(ParamMode::Var) => "VAR ",
                Some(ParamMode::In) => "IN ",
                Some(ParamMode::Out) => "OUT ",
                None => "",
            };
            format!(
                "{}{}: {}",
                mode,
                parameter.names.join(", "),
                render_semantic_type(&parameter.ty)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let result = signature
        .result_type
        .as_ref()
        .map(|item| format!(": {}", render_semantic_type(item)))
        .unwrap_or_default();
    let attributes = {
        let mut parts = Vec::new();
        if signature.is_new {
            parts.push("NEW".to_string());
        }
        if let Some(flavor) = signature.flavor {
            parts.push(match flavor {
                MethodFlavor::Abstract => "ABSTRACT".to_string(),
                MethodFlavor::Empty => "EMPTY".to_string(),
                MethodFlavor::Extensible => "EXTENSIBLE".to_string(),
            });
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(", {}", parts.join(", "))
        }
    };
    format!("PROCEDURE {}({}){}{}", receiver, parameters, result, attributes)
}

fn procedure_types_match(expected: &ProcedureType, actual: &ProcedureType) -> bool {
    expected.result_type == actual.result_type
        && expected.parameters.len() == actual.parameters.len()
        && expected
            .parameters
            .iter()
            .zip(&actual.parameters)
            .all(|(left, right)| left.mode == right.mode && left.ty == right.ty)
}

fn has_out_parameter(parameters: &FormalParameters) -> bool {
    parameters
        .sections
        .iter()
        .any(|section| section.mode == Some(ParamMode::Out))
}

fn method_flavor_name(flavor: MethodFlavor) -> &'static str {
    match flavor {
        MethodFlavor::Abstract => "abstract",
        MethodFlavor::Empty => "empty",
        MethodFlavor::Extensible => "extensible",
    }
}

fn export_mark_name(export: ExportMark) -> &'static str {
    match export {
        ExportMark::Exported => "*",
        ExportMark::ReadOnly => "-",
    }
}

fn module_procedure_identity(heading: &newcp_parser::ProcedureHeading) -> String {
    match &heading.receiver {
        Some(receiver) => format!("method:{}:{}", receiver.ty, heading.name.name),
        None => format!("procedure:{}", heading.name.name),
    }
}

impl<'a> Analyzer<'a> {
    fn validate_builtin_call(
        &self,
        proc: BuiltinProc,
        args: &[Expr],
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        match proc {
            BuiltinProc::New => {
                if args.is_empty() {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        0,
                        0,
                        "NEW requires at least one argument".to_string(),
                    ));
                    return;
                }
                self.validate_builtin_var_designator(
                    "NEW",
                    1,
                    &args[0],
                    procedure_name,
                    local_symbols,
                    diagnostics,
                );
                if let Some(arg_type) = self.infer_expr_type(&args[0], local_symbols, scope_type_names) {
                    let resolved = self.resolve_named_type_one_level(&arg_type, local_symbols);
                    match &resolved {
                        SemanticType::Pointer { target, .. } => {
                            // Check that the pointed-to type is not an ABSTRACT record
                            let target_resolved = self.resolve_named_type_one_level(target, local_symbols);
                            if matches!(target_resolved, SemanticType::Record { flavor: Some(RecordFlavor::Abstract), .. }) {
                                let (line, column) = expr_position(&args[0]);
                                diagnostics.push(make_diagnostic(
                                    procedure_name,
                                    line,
                                    column,
                                    "NEW cannot allocate an ABSTRACT record type".to_string(),
                                ));
                            }
                        }
                        _ => {
                            let (line, column) = expr_position(&args[0]);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                format!("NEW first argument must be a pointer variable, found {}", render_semantic_type(&arg_type)),
                            ));
                        }
                    }
                }
            }
            BuiltinProc::SystemAdr => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics);
            }
            BuiltinProc::SystemVal => {
                self.require_builtin_exact_arity(proc, args, 2, procedure_name, diagnostics);
                if args.len() == 2 {
                    if !matches!(&args[0], Expr::Designator(designator) if self.designator_denotes_type_name(designator, local_symbols)) {
                        let (line, column) = expr_position(&args[0]);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            "VAL argument 1 must be a type".to_string(),
                        ));
                    }
                    if let Some(target_type) = self.infer_expr_type(&args[0], local_symbols, scope_type_names)
                        && self.is_managed_pointer_type(&target_type, local_symbols)
                    {
                        let (line, column) = expr_position(&args[0]);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            "SYSTEM.VAL may not produce a managed pointer; use POINTER [untagged]".to_string(),
                        ));
                    }
                }
            }
            BuiltinProc::SystemLsh | BuiltinProc::SystemRot => {
                self.require_builtin_exact_arity(proc, args, 2, procedure_name, diagnostics);
                if args.len() == 2 {
                    self.require_builtin_integer_arg(proc, 1, &args[0], procedure_name, local_symbols, scope_type_names, diagnostics);
                    self.require_builtin_integer_arg(proc, 2, &args[1], procedure_name, local_symbols, scope_type_names, diagnostics);
                }
            }
            BuiltinProc::SystemTyp => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics);
            }
            BuiltinProc::SystemBit | BuiltinProc::SystemGetReg | BuiltinProc::SystemPutReg => {
                let (line, column) = args.first().map(expr_position).unwrap_or((0, 0));
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!("SYSTEM.{} is x86-32 specific and not supported on this target", proc.name()),
                ));
            }
            BuiltinProc::SystemGet | BuiltinProc::SystemPut => {
                self.require_builtin_exact_arity(proc, args, 2, procedure_name, diagnostics);
                if args.len() == 2 {
                    self.require_builtin_integer_arg(proc, 1, &args[0], procedure_name, local_symbols, scope_type_names, diagnostics);
                    if proc == BuiltinProc::SystemGet {
                        self.validate_builtin_var_designator(proc.name(), 2, &args[1], procedure_name, local_symbols, diagnostics);
                    }
                    if let Some(arg_type) = self.infer_expr_type(&args[1], local_symbols, scope_type_names)
                        && self.is_managed_pointer_type(&arg_type, local_symbols)
                    {
                        let (line, column) = expr_position(&args[1]);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!("SYSTEM.{} may not read or write a managed pointer", proc.name()),
                        ));
                    }
                }
            }
            BuiltinProc::SystemMove => {
                self.require_builtin_exact_arity(proc, args, 3, procedure_name, diagnostics);
                if args.len() == 3 {
                    self.require_builtin_integer_arg(proc, 1, &args[0], procedure_name, local_symbols, scope_type_names, diagnostics);
                    self.require_builtin_integer_arg(proc, 2, &args[1], procedure_name, local_symbols, scope_type_names, diagnostics);
                    self.require_builtin_integer_arg(proc, 3, &args[2], procedure_name, local_symbols, scope_type_names, diagnostics);
                }
            }
            BuiltinProc::SystemNew => {
                self.require_builtin_exact_arity(proc, args, 2, procedure_name, diagnostics);
                if args.len() == 2 {
                    self.validate_builtin_var_designator(proc.name(), 1, &args[0], procedure_name, local_symbols, diagnostics);
                    self.require_builtin_integer_arg(proc, 2, &args[1], procedure_name, local_symbols, scope_type_names, diagnostics);
                    if let Some(arg_type) = self.infer_expr_type(&args[0], local_symbols, scope_type_names) {
                        match self.resolve_named_type_one_level(&arg_type, local_symbols) {
                            SemanticType::Pointer { untagged: true, .. } => {}
                            other => {
                                let (line, column) = expr_position(&args[0]);
                                diagnostics.push(make_diagnostic(
                                    procedure_name,
                                    line,
                                    column,
                                    format!("SYSTEM.NEW first argument must be a POINTER [untagged] variable, found {}", render_semantic_type(&other)),
                                ));
                            }
                        }
                    }
                }
            }
            BuiltinProc::Abs => self.require_builtin_numeric_args(
                proc,
                args,
                1,
                procedure_name,
                local_symbols,
                scope_type_names,
                diagnostics,
            ),
            BuiltinProc::Ash => {
                self.require_builtin_exact_arity(proc, args, 2, procedure_name, diagnostics);
                if args.len() == 2 {
                    self.require_builtin_integer_arg(
                        proc,
                        1,
                        &args[0],
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                    self.require_builtin_integer_arg(
                        proc,
                        2,
                        &args[1],
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                }
            }
            BuiltinProc::Bits | BuiltinProc::Chr | BuiltinProc::Odd => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics);
                if args.len() == 1 {
                    self.require_builtin_integer_arg(
                        proc,
                        1,
                        &args[0],
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                }
            }
            BuiltinProc::Cap => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics);
                if args.len() == 1
                    && let Some(arg_type) =
                        self.infer_expr_type(&args[0], local_symbols, scope_type_names)
                    && !is_character_like_type(&arg_type)
                {
                    let (line, column) = expr_position(&args[0]);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!(
                            "CAP argument 1 must be a character type, found {}",
                            render_semantic_type(&arg_type)
                        ),
                    ));
                }
            }
            BuiltinProc::Inc | BuiltinProc::Dec => {
                if !(1..=2).contains(&args.len()) {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        0,
                        0,
                        format!("{} expects 1 or 2 arguments", proc.name()),
                    ));
                    return;
                }
                self.validate_builtin_var_designator(
                    proc.name(),
                    1,
                    &args[0],
                    procedure_name,
                    local_symbols,
                    diagnostics,
                );
            }
            BuiltinProc::Incl | BuiltinProc::Excl => {
                if args.len() != 2 {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        0,
                        0,
                        format!("{} expects 2 arguments", proc.name()),
                    ));
                    return;
                }
                self.validate_builtin_var_designator(
                    proc.name(),
                    1,
                    &args[0],
                    procedure_name,
                    local_symbols,
                    diagnostics,
                );
                // First arg must be SET
                if let Some(arg_type) = self.infer_expr_type(&args[0], local_symbols, scope_type_names) {
                    if !matches!(arg_type, SemanticType::Builtin(BuiltinType::Set)) {
                        let (line, column) = expr_position(&args[0]);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!("{} argument 1 must be SET, found {}", proc.name(), render_semantic_type(&arg_type)),
                        ));
                    }
                }
                // Second arg must be integer
                self.require_builtin_integer_arg(
                    proc,
                    2,
                    &args[1],
                    procedure_name,
                    local_symbols,
                    scope_type_names,
                    diagnostics,
                );
            }
            BuiltinProc::Entier => self.require_builtin_real_args(
                proc,
                args,
                1,
                procedure_name,
                local_symbols,
                scope_type_names,
                diagnostics,
            ),
            BuiltinProc::Len => {
                if !(1..=2).contains(&args.len()) {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        0,
                        0,
                        "LEN expects 1 or 2 arguments".to_string(),
                    ));
                } else if let Some(arg_type) = self.infer_expr_type(&args[0], local_symbols, scope_type_names) {
                    let is_valid = matches!(
                        &arg_type,
                        SemanticType::Array { .. }
                            | SemanticType::Builtin(BuiltinType::String)
                            | SemanticType::Builtin(BuiltinType::ShortString)
                    ) || matches!(&arg_type, SemanticType::Named { .. });
                    if !is_valid {
                        let (line, column) = expr_position(&args[0]);
                        diagnostics.push(make_diagnostic(
                            procedure_name,
                            line,
                            column,
                            format!(
                                "LEN argument 1 must be an array or string type, found {}",
                                render_semantic_type(&arg_type)
                            ),
                        ));
                    }
                }
            }
            BuiltinProc::Long | BuiltinProc::Short => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics);
            }
            BuiltinProc::Max | BuiltinProc::Min => {
                if !(1..=2).contains(&args.len()) {
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        0,
                        0,
                        format!("{} expects 1 or 2 arguments", proc.name()),
                    ));
                }
            }
            BuiltinProc::Ord => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics);
                if args.len() == 1
                    && let Some(arg_type) =
                        self.infer_expr_type(&args[0], local_symbols, scope_type_names)
                    && !matches!(
                        arg_type,
                        SemanticType::Builtin(BuiltinType::Byte)
                            | SemanticType::Builtin(BuiltinType::Char)
                            | SemanticType::Builtin(BuiltinType::ShortChar)
                            | SemanticType::Builtin(BuiltinType::Set)
                    )
                {
                    let (line, column) = expr_position(&args[0]);
                    diagnostics.push(make_diagnostic(
                        procedure_name,
                        line,
                        column,
                        format!(
                            "ORD argument 1 must be BYTE, CHAR, SHORTCHAR, or SET, found {}",
                            render_semantic_type(&arg_type)
                        ),
                    ));
                }
            }
            BuiltinProc::Size => {
                self.require_builtin_exact_arity(proc, args, 1, procedure_name, diagnostics)
            }
            BuiltinProc::Assert | BuiltinProc::Halt => {
                // ASSERT(condition) or ASSERT(condition, code): first arg must be BOOLEAN
                if proc == BuiltinProc::Assert && !args.is_empty() {
                    if let Some(arg_type) = self.infer_expr_type(&args[0], local_symbols, scope_type_names) {
                        if !is_boolean_type(&arg_type) {
                            let (line, column) = expr_position(&args[0]);
                            diagnostics.push(make_diagnostic(
                                procedure_name,
                                line,
                                column,
                                format!("ASSERT condition must be BOOLEAN, found {}", render_semantic_type(&arg_type)),
                            ));
                        }
                    }
                }
            }
        }
    }

    fn require_builtin_exact_arity(
        &self,
        proc: BuiltinProc,
        args: &[Expr],
        expected: usize,
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        if args.len() != expected {
            diagnostics.push(make_diagnostic(
                procedure_name,
                0,
                0,
                format!(
                    "{} expects {} argument{}",
                    proc.name(),
                    expected,
                    if expected == 1 { "" } else { "s" }
                ),
            ));
        }
    }

    fn require_builtin_integer_arg(
        &self,
        proc: BuiltinProc,
        index: usize,
        expr: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        if let Some(arg_type) = self.infer_expr_type(expr, local_symbols, scope_type_names)
            && !is_integer_type(&arg_type)
        {
            let (line, column) = expr_position(expr);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!(
                    "{} argument {} must be an integer type, found {}",
                    proc.name(),
                    index,
                    render_semantic_type(&arg_type)
                ),
            ));
        }
    }

    fn require_builtin_numeric_args(
        &self,
        proc: BuiltinProc,
        args: &[Expr],
        expected: usize,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        self.require_builtin_exact_arity(proc, args, expected, procedure_name, diagnostics);
        for (index, arg) in args.iter().enumerate() {
            if let Some(arg_type) = self.infer_expr_type(arg, local_symbols, scope_type_names)
                && !is_numeric_type(&arg_type)
            {
                let (line, column) = expr_position(arg);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "{} argument {} must be a numeric type, found {}",
                        proc.name(),
                        index + 1,
                        render_semantic_type(&arg_type)
                    ),
                ));
            }
        }
    }

    fn require_builtin_real_args(
        &self,
        proc: BuiltinProc,
        args: &[Expr],
        expected: usize,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        scope_type_names: &HashSet<String>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        self.require_builtin_exact_arity(proc, args, expected, procedure_name, diagnostics);
        for (index, arg) in args.iter().enumerate() {
            if let Some(arg_type) = self.infer_expr_type(arg, local_symbols, scope_type_names)
                && !matches!(
                    arg_type,
                    SemanticType::Builtin(BuiltinType::ShortReal)
                        | SemanticType::Builtin(BuiltinType::Real)
                )
            {
                let (line, column) = expr_position(arg);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "{} argument {} must be a real type, found {}",
                        proc.name(),
                        index + 1,
                        render_semantic_type(&arg_type)
                    ),
                ));
            }
        }
    }

    fn validate_builtin_var_designator(
        &self,
        builtin_name: &str,
        index: usize,
        actual_expr: &Expr,
        procedure_name: Option<&str>,
        local_symbols: &[SemanticSymbol],
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
        let Expr::Designator(designator) = actual_expr else {
            let (line, column) = expr_position(actual_expr);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("argument {} for {} must be an assignable designator", index, builtin_name),
            ));
            return;
        };

        if designator
            .selectors
            .iter()
            .any(|selector| matches!(selector, Selector::Call(_) | Selector::TypeGuard(_) | Selector::AmbiguousParen(_)))
        {
            let (line, column) = designator_position(designator);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("argument {} for {} is not assignable", index, builtin_name),
            ));
            return;
        }

        if let Some(symbol) = self.lookup_symbol(&designator.base.name, local_symbols) {
            if !matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Parameter | SymbolKind::Receiver) {
                let (line, column) = designator_position(designator);
                diagnostics.push(make_diagnostic(
                    procedure_name,
                    line,
                    column,
                    format!(
                        "argument {} for {} must name a variable, parameter, or receiver, found {} {}",
                        index,
                        builtin_name,
                        render_symbol_kind(symbol.kind),
                        symbol.name
                    ),
                ));
            }
        }
    }
}

fn render_optional_module_name(module: &Option<String>, name: &str) -> String {
    match module {
        Some(module) => format!("{}.{}", module, name),
        None => name.to_string(),
    }
}

fn render_qualident(qualident: &QualIdent) -> String {
    render_optional_module_name(&qualident.module, &qualident.name)
}

fn render_designator(designator: &Designator) -> String {
    let mut text = render_qualident(&designator.base);
    for selector in &designator.selectors {
        match selector {
            Selector::Field(name) => {
                text.push('.');
                text.push_str(name);
            }
            Selector::Index(items) => {
                text.push('[');
                text.push_str(&items.iter().map(render_expr).collect::<Vec<_>>().join(", "));
                text.push(']');
            }
            Selector::Dereference => text.push('^'),
            Selector::TypeGuard(guard) => {
                text.push('(');
                text.push_str(&render_qualident(guard));
                text.push(')');
            }
            Selector::Call(args) => {
                text.push('(');
                text.push_str(&args.iter().map(render_expr).collect::<Vec<_>>().join(", "));
                text.push(')');
            }
            Selector::AmbiguousParen(guard) => {
                text.push('(');
                text.push_str(&render_qualident(guard));
                text.push(')');
                text.push('?');
            }
            Selector::StringDereference => text.push('$'),
        }
    }
    text
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value, .. } => match value {
            newcp_parser::Literal::Integer(value)
            | newcp_parser::Literal::Real(value)
            | newcp_parser::Literal::Character(value)
            | newcp_parser::Literal::String(value) => value.clone(),
        },
        Expr::Nil { .. } => "NIL".to_string(),
        Expr::Designator(designator) => render_designator(designator),
        Expr::Set { elements, .. } => format!(
            "{{{}}}",
            elements
                .iter()
                .map(|item| match &item.end {
                    Some(end) => format!("{}..{}", render_expr(&item.start), render_expr(end)),
                    None => render_expr(&item.start),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Unary { op, expr, .. } => format!(
            "{}{}",
            match op {
                newcp_parser::UnaryOp::Plus => "+",
                newcp_parser::UnaryOp::Minus => "-",
                newcp_parser::UnaryOp::Not => "~",
            },
            render_expr(expr)
        ),
        Expr::Binary { left, op, right, .. } => format!(
            "({} {} {})",
            render_expr(left),
            render_binary_op(*op),
            render_expr(right)
        ),
    }
}

fn render_binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Or => "OR",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Div => "DIV",
        BinaryOp::Mod => "MOD",
        BinaryOp::And => "&",
        BinaryOp::Equal => "=",
        BinaryOp::NotEqual => "#",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::In => "IN",
        BinaryOp::Is => "IS",
    }
}

fn render_unary_op(op: newcp_parser::UnaryOp) -> &'static str {
    match op {
        newcp_parser::UnaryOp::Plus => "+",
        newcp_parser::UnaryOp::Minus => "-",
        newcp_parser::UnaryOp::Not => "~",
    }
}

fn render_symbol_kind(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Import => "import",
        SymbolKind::Constant => "constant",
        SymbolKind::Type => "type",
        SymbolKind::Variable => "variable",
        SymbolKind::Procedure => "procedure",
        SymbolKind::Parameter => "parameter",
        SymbolKind::Receiver => "receiver",
    }
}

fn render_param_mode(mode: ParamMode) -> &'static str {
    match mode {
        ParamMode::Var => "VAR",
        ParamMode::In => "IN",
        ParamMode::Out => "OUT",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParameterBinding {
    mode: Option<ParamMode>,
    ty: SemanticType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeTestContext<'a> {
    IsExpr,
    WithGuard(&'a str),
    DesignatorGuard,
}

impl TypeTestContext<'_> {
    fn invalid_subject_message(self) -> String {
        match self {
            TypeTestContext::IsExpr => {
                "IS left operand must be a record parameter/receiver or a pointer to record type"
                    .to_string()
            }
            TypeTestContext::WithGuard(name) => format!(
                "WITH guard variable {} must be a record parameter/receiver or a pointer to record type",
                name
            ),
            TypeTestContext::DesignatorGuard => {
                "type guard subject must be a record parameter/receiver or a pointer to record type"
                    .to_string()
            }
        }
    }

    fn invalid_target_message(self, target: &str) -> String {
        match self {
            TypeTestContext::IsExpr => {
                format!("IS target {} must name a record type", target)
            }
            TypeTestContext::WithGuard(_) => {
                format!("WITH guard type {} must name a record type", target)
            }
            TypeTestContext::DesignatorGuard => {
                format!("type guard target {} must name a record type", target)
            }
        }
    }

    fn non_extension_message(self, target: &str, static_type: &str) -> String {
        match self {
            TypeTestContext::IsExpr => {
                format!("IS target {} must extend {}", target, static_type)
            }
            TypeTestContext::WithGuard(_) => {
                format!("WITH guard type {} must extend {}", target, static_type)
            }
            TypeTestContext::DesignatorGuard => {
                format!("type guard target {} must extend {}", target, static_type)
            }
        }
    }
}

fn flatten_parameter_bindings(parameters: &[ParameterType]) -> Vec<ParameterBinding> {
    let mut flattened = Vec::new();
    for parameter in parameters {
        for _ in &parameter.names {
            flattened.push(ParameterBinding {
                mode: parameter.mode,
                ty: parameter.ty.clone(),
            });
        }
    }
    flattened
}

#[cfg(test)]
mod tests {
    use super::*;
    use newcp_parser::parse_module_ast;

    #[test]
    fn sema_dump_reports_export_categories() {
        let temp = std::env::temp_dir().join("newcp-sema-test.cp");
        std::fs::write(
            &temp,
            "MODULE Demo;\nIMPORT Kernel;\nCONST Version* = 1;\nVAR Current* : INTEGER;\nPROCEDURE Run*;\nBEGIN\nEND Run;\nEND Demo.",
        )
        .expect("write test module");

        let dump = dump_sema(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("module: Demo"), "missing module name\n{dump}");
        assert!(dump.contains("const *Version"), "missing Version constant\n{dump}");
        assert!(dump.contains("var   *Current"), "missing Current variable\n{dump}");
        assert!(dump.contains("proc  *Run"), "missing Run procedure\n{dump}");
        assert!(dump.contains("var   *Current : INTEGER"), "missing Current type\n{dump}");
    }

    #[test]
    fn sema_resolves_builtin_and_user_defined_types_canonically() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Tree = POINTER TO Node; Node = RECORD key: INTEGER END;\nVAR root: Tree; current: INTEGER;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let tree = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Tree" && symbol.kind == SymbolKind::Type)
            .and_then(|symbol| symbol.declared_type.as_ref())
            .expect("Tree type symbol");
        assert_eq!(
            tree,
            &SemanticType::Pointer {
                target: Box::new(SemanticType::Named {
                    module: None,
                    name: "Node".to_string(),
                    kind: NamedTypeKind::UserDefined,
                }),
                untagged: false,
            }
        );

        let current = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "current")
            .and_then(|symbol| symbol.declared_type.as_ref())
            .expect("current variable type");
        assert_eq!(current, &SemanticType::Builtin(BuiltinType::Integer));
    }

    #[test]
    fn sema_captures_procedure_signatures_as_types() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run*(x: INTEGER): BOOLEAN;\nBEGIN\nRETURN x = 1\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let run = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Run" && symbol.kind == SymbolKind::Procedure)
            .and_then(|symbol| symbol.declared_type.as_ref())
            .expect("procedure type");
        let SemanticType::Procedure(signature) = run else {
            panic!("expected procedure type");
        };
        assert_eq!(signature.parameters.len(), 1);
        assert_eq!(signature.parameters[0].ty, SemanticType::Builtin(BuiltinType::Integer));
        assert_eq!(
            signature.result_type.as_deref(),
            Some(&SemanticType::Builtin(BuiltinType::Boolean))
        );
    }

    #[test]
    fn sema_resolves_ambiguous_parenthesized_selector_against_type_names() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE T = RECORD END;\nPROCEDURE Run;\nBEGIN\nnode(T)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(sema.selector_resolutions.iter().any(|item| {
            item.selector == "T" && item.kind == SelectorResolutionKind::TypeGuard
        }));
    }

    #[test]
    fn sema_leaves_ambiguous_parenthesized_selector_unresolved_without_type_symbol() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nBEGIN\nnode(T)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(sema.selector_resolutions.iter().any(|item| {
            item.selector == "T" && item.kind == SelectorResolutionKind::Unresolved
        }));
    }

    #[test]
    fn sema_reports_assignment_and_return_type_mismatches() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run*(flag: BOOLEAN): BOOLEAN;\nVAR count: INTEGER;\nBEGIN\ncount := flag;\nRETURN count\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("assignment type mismatch"))
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("return type mismatch"))
        );
    }

    #[test]
    fn sema_reports_non_boolean_conditions() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nBEGIN\nIF 1 THEN END;\nWHILE 2 DO END;\nREPEAT UNTIL 3\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("IF condition must be BOOLEAN"))
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("WHILE condition must be BOOLEAN"))
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("REPEAT condition must be BOOLEAN"))
        );
    }

    #[test]
    fn sema_rejects_invalid_case_statements_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nVAR i, j: INTEGER; ch: CHAR; flag: BOOLEAN;\nBEGIN\nCASE flag OF 1: i := 0 END;\nCASE i OF j, 2..1, 1, 1: i := 0 END;\nCASE ch OF 1: i := 0 END\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("CASE expression must be an integer or character type")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("CASE label must be a constant expression")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("CASE label range start must not be greater than its end")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("CASE label 1 overlaps earlier label 1")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("CASE label type INTEGER is not compatible with selector type CHAR")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_invalid_with_guards_and_is_tests_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = RECORD END; Child = RECORD (Base) END; Other = RECORD END; Ptr = POINTER TO Base;\nPROCEDURE Check(IN rec: Base; p: Ptr; flag: BOOLEAN);\nBEGIN\nIF rec IS Child THEN END;\nIF p IS Child THEN END;\nIF flag IS Child THEN END;\nIF p IS flag THEN END;\nIF rec IS Other THEN END;\nWITH rec: Child DO END;\nWITH p: Child DO END;\nWITH flag: Child DO END;\nWITH rec: Other DO END\nEND Check;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains(
                "IS left operand must be a record parameter/receiver or a pointer to record type"
            )),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("IS right operand must be a type identifier")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("IS target Other must extend type:Base")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains(
                "WITH guard variable flag must be a record parameter/receiver or a pointer to record type"
            )),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("WITH guard type Other must extend type:Base")),
            "{}",
            messages
        );
        assert!(
            !sema.diagnostics
                .iter()
                .any(|item| item.message.contains("Child must extend")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_invalid_designator_type_guards_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = RECORD END; Child = RECORD (Base) END; Other = RECORD END; Ptr = POINTER TO Base;\nPROCEDURE Check(IN rec: Base; p: Ptr; flag: BOOLEAN);\nBEGIN\nrec(Child);\np(Child);\nrec(Other);\nflag(Child)\nEND Check;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("type guard subject must be a record parameter/receiver or a pointer to record type")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("type guard target Other must extend type:Base")),
            "{}",
            messages
        );
        assert!(
            !sema.diagnostics
                .iter()
                .any(|item| item.message.contains("type guard target Child must extend")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_tags_simd_candidate_shapes_for_records_and_arrays() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Pair = RECORD x, y: REAL END; Vec4 = RECORD x, y, z, w: SHORTREAL END;\nVAR pairs: ARRAY 32 OF Pair; vecs: ARRAY 32 OF Vec4; scalars: ARRAY 32 OF REAL;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);

        let pair = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Pair")
            .and_then(|symbol| symbol.simd_shape.as_ref());
        assert_eq!(
            pair,
            Some(&SimdShape {
                layout: SimdLayout::PackedRecord,
                lane_kind: SimdLaneKind::Float64,
                lane_count: 2,
                packed_bytes: 16,
            })
        );

        let vecs = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "vecs")
            .and_then(|symbol| symbol.simd_shape.as_ref());
        assert_eq!(
            vecs,
            Some(&SimdShape {
                layout: SimdLayout::ArrayOfStruct,
                lane_kind: SimdLaneKind::Float32,
                lane_count: 4,
                packed_bytes: 16,
            })
        );

        let scalars = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "scalars")
            .and_then(|symbol| symbol.simd_shape.as_ref());
        assert_eq!(
            scalars,
            Some(&SimdShape {
                layout: SimdLayout::ScalarArray,
                lane_kind: SimdLaneKind::Float64,
                lane_count: 1,
                packed_bytes: 8,
            })
        );
    }

    #[test]
    fn sema_dump_renders_simd_layout_kinds() {
        let temp = std::env::temp_dir().join("newcp-sema-simd-dump.cp");
        std::fs::write(
            &temp,
            "MODULE Demo;\nTYPE Pair = RECORD x, y: REAL END;\nVAR pairs: ARRAY 32 OF Pair;\nEND Demo.",
        )
        .expect("write test module");

        let dump = dump_sema(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("type   Pair") && dump.contains("packed-record:f64x2:16B"), "missing Pair simd shape\n{dump}");
        assert!(dump.contains("var    pairs") && dump.contains("array-of-struct:f64x2:16B"), "missing pairs simd shape\n{dump}");
    }

    #[test]
    fn sema_rejects_invalid_designators_and_calls_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE F(x: INTEGER);\nBEGIN\nEND F;\nPROCEDURE Run;\nVAR i: INTEGER; p: POINTER TO INTEGER; arr: ARRAY 4 OF INTEGER; recs: ARRAY 4 OF RECORD y: INTEGER END;\nBEGIN\nmissing := 1;\nrecs[0].x := 1;\ni[TRUE] := 1;\ni^ := 1;\ni();\nF(~FALSE)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("identifier missing is not declared")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("field x does not exist on RECORD")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("index selector requires an array")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("dereference requires a pointer")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("call selector requires a procedure")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("argument 1 type mismatch")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_duplicate_declarations_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nIMPORT IO;\nCONST IO = 1;\nVAR x: INTEGER; x: INTEGER;\nPROCEDURE Run(a: INTEGER; a: INTEGER);\nVAR y: INTEGER; y: INTEGER;\nBEGIN\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate module-scope declaration: IO")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate module-scope declaration: x")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate procedure-scope declaration: a")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate procedure-scope declaration: y")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_unknown_type_names_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Box = POINTER TO Missing;\nPROCEDURE Run(x: Missing): Missing;\nVAR local: Missing;\nBEGIN\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("unknown type Missing")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_system_usage_without_import() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Raw = RECORD [untagged] value: INTEGER END;\nVAR x: INTEGER;\nPROCEDURE Run;\nBEGIN\n  x := SYSTEM.ADR(x)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("requires IMPORT SYSTEM")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_accepts_system_new_on_untagged_pointer() {
        let module = parse_module_ast(
            "MODULE Demo;\nIMPORT SYSTEM;\nTYPE Raw = RECORD [untagged] value: INTEGER END;\nTYPE RawPtr = POINTER [untagged] TO Raw;\nVAR p: RawPtr;\nPROCEDURE Run;\nBEGIN\n  SYSTEM.NEW(p, 64)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(sema.diagnostics.is_empty(), "{}", messages);
    }

    #[test]
    fn sema_rejects_invalid_operators_and_assignment_targets_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nCONST Flag = TRUE;\nTYPE T = RECORD END;\nPROCEDURE P;\nBEGIN\nEND P;\nPROCEDURE Run;\nVAR i: INTEGER; s: SET;\nBEGIN\nFlag := FALSE;\nP := P;\ni := ~i;\ni := i OR 1;\ni := i IN s;\ni := s / s;\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("assignment target must be a variable, parameter, or receiver, found constant Flag")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("assignment target must be a variable, parameter, or receiver, found procedure P")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("invalid unary operator ~ for INTEGER")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("invalid operands for OR: INTEGER and INTEGER")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("assignment type mismatch: expected INTEGER, found BOOLEAN")),
            "{}",
            messages
        );
        // SET / SET is now valid (symmetric difference); the error should be the
        // assignment type mismatch: i (INTEGER) := s/s (SET)
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("assignment type mismatch") && item.message.contains("SET")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_var_and_out_arguments_that_are_not_assignable() {
        let module = parse_module_ast(
            "MODULE Demo;\nCONST Flag = 1;\nPROCEDURE Mutate(VAR x: INTEGER; OUT y: INTEGER; IN z: INTEGER);\nBEGIN\nEND Mutate;\nPROCEDURE Run;\nVAR value: INTEGER;\nBEGIN\nMutate(Flag, value + 1, 1);\nMutate(value, 0, 1)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("argument 1 for VAR parameter must name a variable, parameter, or receiver, found constant Flag")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("argument 2 for OUT parameter must be an assignable designator")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_invalid_for_by_step_early() {
        // zero literal, variable step, and zero named constant are all rejected
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nCONST Zero = 0;\nVAR i, n: INTEGER;\nBEGIN\nFOR i := 0 TO 10 BY 0 DO END;\nFOR i := 0 TO 10 BY n DO END;\nFOR i := 0 TO 10 BY Zero DO END\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("FOR step must be nonzero")),
            "expected nonzero error: {}",
            messages
        );
        assert_eq!(
            sema.diagnostics
                .iter()
                .filter(|item| item.message.contains("FOR step must be nonzero"))
                .count(),
            2,
            "expected exactly 2 nonzero errors (literal 0 and named const Zero): {}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("FOR step must be a constant expression")),
            "expected constant expression error: {}",
            messages
        );
    }

    #[test]
    fn sema_accepts_valid_for_by_steps() {
        // positive literal, negative literal, and named nonzero constant are all valid
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nCONST Step = 2;\nVAR i: INTEGER;\nBEGIN\nFOR i := 0 TO 10 BY 1 DO END;\nFOR i := 10 TO 0 BY -1 DO END;\nFOR i := 0 TO 100 BY Step DO END\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let by_errors: Vec<_> = sema
            .diagnostics
            .iter()
            .filter(|item| item.message.contains("FOR step"))
            .collect();
        assert!(
            by_errors.is_empty(),
            "expected no FOR step errors: {}",
            by_errors
                .iter()
                .map(|item| item.message.as_str())
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }

    #[test]
    fn sema_rejects_forward_declaration_mismatches_early() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE ^ Missing(x: INTEGER);\nPROCEDURE ^ Mismatch(x: INTEGER);\nPROCEDURE Mismatch(x: BOOLEAN);\nBEGIN\nEND Mismatch;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("forward declaration for Missing has no matching implementation")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("forward declaration for Mismatch does not match implementation")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_accepts_builtin_constants_and_reports_module_body_errors() {
        let module = parse_module_ast(
            "MODULE Demo;\nBEGIN\nTRUE := FALSE;\nNEW(x)\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("assignment target must be a variable, parameter, or receiver, found constant TRUE")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("identifier x is not declared")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_types_strings_sets_and_longint_literals_more_precisely() {
        let module = parse_module_ast(
            "MODULE Demo;\nCONST Big = 0FFFF0000L;\nPROCEDURE Run;\nVAR s: ARRAY 8 OF CHAR; setA, setB: SET;\nBEGIN\ns := 'ok';\nsetA := setA + setB;\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let big = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Big")
            .expect("Big constant exists");
        assert_eq!(big.const_value, Some(ConstValue::Integer(4294901760)));
        assert_eq!(
            big.declared_type,
            Some(SemanticType::Builtin(BuiltinType::LongInt)),
            "explicit L-suffix constant stores LONGINT declared_type"
        );

        let set_errors: Vec<_> = sema
            .diagnostics
            .iter()
            .filter(|item| item.message.contains("SET and SET"))
            .collect();
        assert!(set_errors.is_empty(), "unexpected set operator errors");
    }

    #[test]
    fn sema_accepts_nil_and_pointer_extension_assignments() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = RECORD END; Child = RECORD (Base) END; Ptr = POINTER TO Base; ChildPtr = POINTER TO Child; Proc = PROCEDURE();\nPROCEDURE Run;\nVAR p: Ptr; cp: ChildPtr; fn: Proc;\nBEGIN\np := cp;\np := NIL;\nfn := NIL\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let mismatch_errors: Vec<_> = sema
            .diagnostics
            .iter()
            .filter(|item| item.message.contains("assignment type mismatch"))
            .collect();
        assert!(mismatch_errors.is_empty(), "unexpected assignment mismatch errors");
    }

    #[test]
    fn sema_resolves_inherited_fields_and_methods_on_records() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = RECORD x: INTEGER END; Child = RECORD (Base) END;\nPROCEDURE (self: Base) Ping(y: INTEGER): BOOLEAN, NEW;\nBEGIN\nRETURN TRUE\nEND Ping;\nPROCEDURE Run;\nVAR child: Child; ok: BOOLEAN;\nBEGIN\nchild.x := 1;\nok := child.Ping(1)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            !sema.diagnostics.iter().any(|item| item.message.contains("field x does not exist")),
            "{}",
            messages
        );
        assert!(
            !sema.diagnostics.iter().any(|item| item.message.contains("field Ping does not exist")),
            "{}",
            messages
        );
        assert!(
            !sema.diagnostics.iter().any(|item| item.message.contains("call selector requires a procedure")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_types_common_builtin_procedure_results() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nVAR i: INTEGER; l: LONGINT; r: REAL; c: CHAR; b: BOOLEAN; s: ARRAY 8 OF CHAR;\nBEGIN\nr := ABS(i);\nl := LONG(i);\ni := SHORT(l);\ni := LEN(s);\ni := ORD(c);\nc := CHR(i);\nb := ODD(i)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let builtin_errors: Vec<_> = sema
            .diagnostics
            .iter()
            .filter(|item| item.message.contains("ABS argument")
                || item.message.contains("LONG argument")
                || item.message.contains("SHORT argument")
                || item.message.contains("LEN")
                || item.message.contains("ORD argument")
                || item.message.contains("CHR argument")
                || item.message.contains("ODD argument")
                || item.message.contains("assignment type mismatch"))
            .collect();
        assert!(builtin_errors.is_empty(), "unexpected builtin typing errors");
    }

    #[test]
    fn sema_recognizes_intshort_and_builtin_conversions() {
        let module = parse_module_ast(
            "MODULE Demo;\nCONST Hex = 7FFFFFFFH;\nVAR Mid*: INTSHORT;\nPROCEDURE Run;\nVAR s: SHORTINT; m: INTSHORT; i: INTEGER; l: LONGINT;\nBEGIN\nMid := Hex;\nm := LONG(s);\ni := LONG(m);\nl := LONG(i);\ni := SHORT(l);\nm := SHORT(i);\ns := SHORT(m)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.is_empty(),
            "unexpected INTSHORT diagnostics: {messages}"
        );

        let mid = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Mid")
            .expect("Mid symbol exists");
        assert_eq!(mid.declared_type, Some(SemanticType::Builtin(BuiltinType::IntShort)));

        let hex = sema
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Hex")
            .expect("Hex constant exists");
        assert_eq!(hex.declared_type, Some(SemanticType::Builtin(BuiltinType::IntShort)));
    }

    #[test]
    fn sema_rejects_invalid_builtin_procedure_usage() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nVAR b: BOOLEAN; c: CHAR;\nBEGIN\nCHR(b);\nORD(b);\nABS(c);\nBITS(c)\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("CHR argument 1 must be an integer type")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("ORD argument 1 must be CHAR, SHORTCHAR, or SET")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("ABS argument 1 must be a numeric type")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("BITS argument 1 must be an integer type")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_invalid_method_contracts() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = EXTENSIBLE RECORD END; Child = RECORD (Base) END; Plain = RECORD END; AbstractBase = ABSTRACT RECORD END; Concrete = RECORD (AbstractBase) END;\nPROCEDURE (b: Base) Grow(), NEW;\nBEGIN\nEND Grow;\nPROCEDURE (c: Child) Grow(), NEW;\nBEGIN\nEND Grow;\nPROCEDURE (p: Plain) Fresh();\nBEGIN\nEND Fresh;\nPROCEDURE (p: Plain) Extend(), NEW, EXTENSIBLE;\nBEGIN\nEND Extend;\nPROCEDURE (p: Plain) Stub(), NEW, EMPTY;\nPROCEDURE (a: AbstractBase) Missing(), NEW, ABSTRACT;\nPROCEDURE Nested;\nPROCEDURE (x: Base) Local(), NEW;\nBEGIN\nEND Local;\nBEGIN\nEND Nested;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("redefining method Grow must not use NEW")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("newly introduced method Fresh must use NEW")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("record Plain must be EXTENSIBLE or ABSTRACT to declare extensible method Extend")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("record Plain must be EXTENSIBLE or ABSTRACT to declare new empty method Stub")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("concrete record Concrete must implement abstract method Missing")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("method Local must be declared at module scope")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_final_method_override_and_signature_mismatch() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = EXTENSIBLE RECORD END; Child = RECORD (Base) END; ExtBase = EXTENSIBLE RECORD END; ExtChild = RECORD (ExtBase) END;\nPROCEDURE (b: Base) Seal(x: INTEGER), NEW;\nBEGIN\nEND Seal;\nPROCEDURE (c: Child) Seal(x: INTEGER);\nBEGIN\nEND Seal;\nPROCEDURE (b: ExtBase) Open(x: INTEGER), NEW, EXTENSIBLE;\nBEGIN\nEND Open;\nPROCEDURE (c: ExtChild) Open(x: BOOLEAN);\nBEGIN\nEND Open;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("method Seal cannot redefine final method declared in Base")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("method Open does not match overridden signature")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_invalid_abstract_and_empty_method_shapes() {
        let mut module = parse_module_ast(
            "MODULE Demo;\nTYPE ExportedBase* = ABSTRACT RECORD END; Base = EXTENSIBLE RECORD END; Child = RECORD (Base) END; AbstractChild = ABSTRACT RECORD (Base) END;\nPROCEDURE (e: ExportedBase) Hidden(), NEW, ABSTRACT;\nPROCEDURE (b: Base) MissingBody(), NEW;\nBEGIN\nEND MissingBody;\nPROCEDURE (b: Base) Concrete(), NEW, EXTENSIBLE;\nBEGIN\nEND Concrete;\nPROCEDURE (b: Base) EmptyResult(): INTEGER, NEW, EMPTY;\nPROCEDURE (b: Base) EmptyOut(OUT x: INTEGER), NEW, EMPTY;\nPROCEDURE (b: Base) EmptyBody(), NEW, EMPTY;\nPROCEDURE (b: Base) AbstractBody(), NEW, ABSTRACT;\nPROCEDURE (b: Base) Open(), NEW, EXTENSIBLE;\nBEGIN\nEND Open;\nPROCEDURE (c: Child) Open(), EMPTY;\nPROCEDURE (a: AbstractChild) Concrete(), ABSTRACT;\nEND Demo.",
        )
        .expect("module should parse");

        for declaration in &mut module.declarations {
            let Declaration::Procedure(procedure) = declaration else {
                continue;
            };
            match procedure.heading.name.name.as_str() {
                "MissingBody" => {
                    procedure.body = None;
                }
                "EmptyBody" | "AbstractBody" => {
                    procedure.body = Some(ProcedureBody {
                        span: procedure.span,
                        declarations: Vec::new(),
                        body: Some(Vec::new()),
                    });
                }
                _ => {}
            }
        }

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("abstract method Hidden of exported record ExportedBase must be exported")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("concrete method MissingBody must have a procedure body")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("empty method EmptyResult must not return a result")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("empty method EmptyOut must not have OUT parameters")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("empty method EmptyBody must not have a procedure body")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("abstract method AbstractBody must not have a procedure body")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("empty method Open may only redefine an empty or abstract method")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("abstract method Concrete may only redefine an abstract method")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_rejects_method_override_export_mismatches() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base* = EXTENSIBLE RECORD END; Child* = RECORD (Base) END;\nPROCEDURE (b: Base) Visible*(), NEW, EXTENSIBLE;\nBEGIN\nEND Visible;\nPROCEDURE (c: Child) Visible();\nBEGIN\nEND Visible;\nPROCEDURE (b: Base) Hidden(), NEW, EXTENSIBLE;\nBEGIN\nEND Hidden;\nPROCEDURE (c: Child) Hidden*();\nBEGIN\nEND Hidden;\nPROCEDURE (b: Base) ReadOnly-(), NEW, EXTENSIBLE;\nBEGIN\nEND ReadOnly;\nPROCEDURE (c: Child) ReadOnly*();\nBEGIN\nEND ReadOnly;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("overriding method Visible of exported record Child must be exported")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("overriding method Hidden must not be exported because the base method is not exported")),
            "{}",
            messages
        );
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("overriding method ReadOnly must use the same export mark as the base method (-)")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_accepts_covariant_pointer_method_results() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = EXTENSIBLE RECORD END; Child = RECORD (Base) END; BasePtr = POINTER TO Base; ChildPtr = POINTER TO Child;\nPROCEDURE (b: Base) Make(): BasePtr, NEW, EXTENSIBLE;\nBEGIN\nRETURN NIL\nEND Make;\nPROCEDURE (c: Child) Make(): ChildPtr;\nBEGIN\nRETURN NIL\nEND Make;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            !sema.diagnostics.iter().any(|item| item.message.contains("does not match overridden signature")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_allows_distinct_methods_with_same_name_on_different_receivers() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = EXTENSIBLE RECORD END; Child = RECORD (Base) END; Sibling = RECORD (Base) END;\nPROCEDURE (c: Child) Ping(), NEW;\nBEGIN\nEND Ping;\nPROCEDURE (s: Sibling) Ping(), NEW;\nBEGIN\nEND Ping;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            !sema
                .diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate module-scope declaration: Ping")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_accepts_valid_forward_declarations_without_duplicates() {
        let module = parse_module_ast(
            "MODULE Demo;\nTYPE Base = EXTENSIBLE RECORD END;\nPROCEDURE ^ Forwarded(x: INTEGER);\nPROCEDURE ^ (b: Base) Make(), NEW, EXTENSIBLE;\nPROCEDURE Forwarded(x: INTEGER);\nBEGIN\nEND Forwarded;\nPROCEDURE (b: Base) Make(), NEW, EXTENSIBLE;\nBEGIN\nEND Make;\nEND Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let messages = sema
            .diagnostics
            .iter()
            .map(|item| item.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        assert!(
            !sema
                .diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate module-scope declaration: Forwarded")),
            "{}",
            messages
        );
        assert!(
            !sema
                .diagnostics
                .iter()
                .any(|item| item.message.contains("duplicate module-scope declaration: Make")),
            "{}",
            messages
        );
        assert!(
            !sema
                .diagnostics
                .iter()
                .any(|item| item.message.contains("forward declaration for")),
            "{}",
            messages
        );
    }

    #[test]
    fn sema_tracks_read_only_export_on_variables() {
        // Variables declared with '-' should have read_only_export = true.
        let module = parse_module_ast(
            "MODULE Demo;\n\
             VAR readWrite*: INTEGER; readOnly-: INTEGER; unexported: INTEGER;\n\
             END Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        let rw = sema.symbols.iter().find(|s| s.name == "readWrite").expect("readWrite");
        let ro = sema.symbols.iter().find(|s| s.name == "readOnly").expect("readOnly");
        let un = sema.symbols.iter().find(|s| s.name == "unexported").expect("unexported");

        assert!(rw.exported, "readWrite should be exported");
        assert!(!rw.read_only_export, "readWrite should not be read-only");
        assert!(ro.exported, "readOnly should be exported");
        assert!(ro.read_only_export, "readOnly should be read-only");
        assert!(!un.exported, "unexported should not be exported");
        assert!(!un.read_only_export, "unexported should not be read-only");
    }

    #[test]
    fn sema_rejects_len_on_non_array() {
        let module = parse_module_ast(
            "MODULE Demo;\n\
             VAR n: INTEGER;\n\
             BEGIN\n\
             n := LEN(n);\n\
             END Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("LEN argument 1 must be an array")),
            "expected LEN type error, got: {}",
            sema.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join(" | ")
        );
    }

    #[test]
    fn sema_rejects_new_on_abstract_record_pointer() {
        let module = parse_module_ast(
            "MODULE Demo;\n\
             TYPE Abs = ABSTRACT RECORD END; AbsPtr = POINTER TO Abs;\n\
             VAR p: AbsPtr;\n\
             BEGIN\n\
             NEW(p);\n\
             END Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("NEW cannot allocate an ABSTRACT record type")),
            "expected NEW abstract error, got: {}",
            sema.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join(" | ")
        );
    }

    #[test]
    fn sema_rejects_incl_with_non_integer_element() {
        let module = parse_module_ast(
            "MODULE Demo;\n\
             VAR s: SET;\n\
             BEGIN\n\
             INCL(s, 3.14);\n\
             END Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("INCL argument 2 must be an integer")),
            "expected INCL integer error, got: {}",
            sema.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join(" | ")
        );
    }

    #[test]
    fn sema_rejects_assert_with_non_boolean_condition() {
        let module = parse_module_ast(
            "MODULE Demo;\n\
             VAR n: INTEGER;\n\
             BEGIN\n\
             ASSERT(n);\n\
             END Demo.",
        )
        .expect("module should parse");

        let sema = analyze_module_ast(&module);
        assert!(
            sema.diagnostics.iter().any(|item| item.message.contains("ASSERT condition must be BOOLEAN")),
            "expected ASSERT boolean error, got: {}",
            sema.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join(" | ")
        );
    }
}
