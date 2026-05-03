use std::collections::HashSet;
use std::path::Path;

use newcp_parser::{
    read_module_ast, BinaryOp, Declaration, Designator, ExportMark, Expr, FPSection, FieldDecl,
    FormalParameters, Guard, MethodFlavor, ModuleAst, ParamMode, ProcedureBody, ProcedureDecl,
    QualIdent, RecordFlavor, Selector, Statement, TypeDecl, TypeExpr,
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
    },
    Record {
        flavor: Option<RecordFlavor>,
        base: Option<Box<SemanticType>>,
        fields: Vec<FieldType>,
        methods: Vec<MethodType>,
    },
    Pointer {
        target: Box<SemanticType>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Integer(i128),
    Real(f64),
    String(String),
    Char(char),
    Boolean(bool),
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
    pub exported: bool,
    pub signature: ProcedureType,
    pub local_symbols: Vec<SemanticSymbol>,
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
    Ok(analyze_module_ast(&module))
}

pub fn analyze_module_ast(module: &ModuleAst) -> SemanticModule {
    let mut analyzer = Analyzer::new(module);
    analyzer.analyze()
}

struct Analyzer<'a> {
    module: &'a ModuleAst,
    module_type_names: HashSet<String>,
    module_symbols: Vec<SemanticSymbol>,
    procedures: Vec<SemanticProcedure>,
    selector_resolutions: Vec<SelectorResolution>,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> Analyzer<'a> {
    fn new(module: &'a ModuleAst) -> Self {
        Self {
            module,
            module_type_names: builtin_type_names(),
            module_symbols: builtin_symbols(),
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
            });
        }

        for declaration in &self.module.declarations {
            match declaration {
                Declaration::Const(item) => {
                    Self::record_identdef_duplicate(&mut scope_names, &item.name, None, "duplicate module-scope declaration", &mut self.diagnostics);
                    let const_value = evaluate_const_expr(&item.value, &[], &self.module_symbols);
                    let declared_type = const_value_type(&const_value);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Constant,
                        exported: item.name.export.is_some(),
                        read_only_export: item.name.export == Some(ExportMark::ReadOnly),
                        declared_type,
                        const_value,
                        simd_shape: None,
                    })
                }
                Declaration::Type(item) => {
                    Self::record_identdef_duplicate(&mut scope_names, &item.name, None, "duplicate module-scope declaration", &mut self.diagnostics);
                    Self::validate_type_expr(&item.ty, &self.module_type_names, None, &mut self.diagnostics);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Type,
                        exported: item.name.export.is_some(),
                        read_only_export: item.name.export == Some(ExportMark::ReadOnly),
                        declared_type: Some(self.resolve_type_decl(
                            Some(&item.name.name),
                            &item.ty,
                            &self.module_type_names,
                        )),
                        const_value: None,
                        simd_shape: None,
                    })
                }
                Declaration::Var(item) => {
                    Self::validate_type_expr(&item.ty, &self.module_type_names, None, &mut self.diagnostics);
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
                    Self::validate_heading_types(&item.heading, &self.module_type_names, None, &mut self.diagnostics);
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
                    Self::validate_heading_types(&item.heading, &self.module_type_names, None, &mut self.diagnostics);
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
                declared_type: Some(self.resolve_receiver_type(receiver.ty.as_str(), &scope_type_names)),
                const_value: None,
                simd_shape: None,
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

        SemanticProcedure {
            name: procedure.heading.name.name.clone(),
            exported: procedure.heading.name.export.is_some(),
            signature,
            local_symbols,
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
                    if !procedure.heading.attributes.is_new {
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
                && self.record_type_info(base_name).map(|(flavor, _)| flavor) != Some(Some(RecordFlavor::Abstract))
            {
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
        Self::validate_type_expr(&section.ty, scope_type_names, procedure_name, diagnostics);
        let declared_type = self.resolve_type_expr(&section.ty, scope_type_names);
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
            });
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
                    let const_value = evaluate_const_expr(&item.value, local_symbols, &self.module_symbols);
                    let declared_type = const_value_type(&const_value);
                    local_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Constant,
                        exported: false,
                        read_only_export: false,
                        declared_type,
                        const_value,
                        simd_shape: None,
                    })
                }
                Declaration::Type(item) => {
                    Self::record_identdef_duplicate(scope_names, &item.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    Self::validate_type_expr(&item.ty, scope_type_names, procedure_name, diagnostics);
                    local_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Type,
                        exported: false,
                        read_only_export: false,
                        declared_type: Some(self.resolve_type_decl(
                            Some(&item.name.name),
                            &item.ty,
                            scope_type_names,
                        )),
                        const_value: None,
                        simd_shape: None,
                    })
                }
                Declaration::Var(item) => {
                    Self::validate_type_expr(&item.ty, scope_type_names, procedure_name, diagnostics);
                    let declared_type = self.resolve_type_expr(&item.ty, scope_type_names);
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
                    Self::validate_heading_types(&item.heading, scope_type_names, procedure_name, diagnostics);
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
                    Self::validate_heading_types(&item.heading, scope_type_names, procedure_name, diagnostics);
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
        procedure_name: Option<&str>,
        diagnostics: &mut Vec<SemanticDiagnostic>,
    ) {
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
                Self::validate_type_expr(&section.ty, scope_type_names, procedure_name, diagnostics);
            }
            if let Some(result) = &parameters.result_type {
                Self::validate_type_expr(result, scope_type_names, procedure_name, diagnostics);
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
            TypeExpr::Array { element_type, .. } => {
                Self::validate_type_expr(element_type, scope_type_names, procedure_name, diagnostics);
            }
            TypeExpr::Record { base, fields, .. } => {
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
                    Self::validate_type_expr(&field.ty, scope_type_names, procedure_name, diagnostics);
                }
            }
            TypeExpr::Pointer { target, .. } => {
                Self::validate_type_expr(target, scope_type_names, procedure_name, diagnostics);
            }
            TypeExpr::Procedure { formal_parameters, .. } => {
                if let Some(parameters) = formal_parameters {
                    for section in &parameters.sections {
                        Self::validate_type_expr(&section.ty, scope_type_names, procedure_name, diagnostics);
                    }
                    if let Some(result) = &parameters.result_type {
                        Self::validate_type_expr(result, scope_type_names, procedure_name, diagnostics);
                    }
                }
            }
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
        self.resolve_type_decl(None, ty, scope_type_names)
    }

    fn resolve_type_decl(
        &self,
        owner_name: Option<&str>,
        ty: &TypeExpr,
        scope_type_names: &HashSet<String>,
    ) -> SemanticType {
        match ty {
            TypeExpr::QualIdent { ident, .. } => self.resolve_named_type(ident, scope_type_names),
            TypeExpr::Array {
                lengths,
                element_type,
                ..
            } => SemanticType::Array {
                lengths: lengths.iter().map(render_expr).collect(),
                element_type: Box::new(self.resolve_type_decl(None, element_type, scope_type_names)),
            },
            TypeExpr::Record {
                flavor,
                base,
                fields,
                ..
            } => SemanticType::Record {
                flavor: *flavor,
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
            TypeExpr::Pointer { target, .. } => SemanticType::Pointer {
                target: Box::new(self.resolve_type_decl(None, target, scope_type_names)),
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
                    .map(|result| Box::new(self.resolve_type_decl(None, result, scope_type_names))),
                is_new: false,
                flavor: None,
            }),
        }
    }

    fn resolve_record_methods(
        &self,
        type_name: &str,
        scope_type_names: &HashSet<String>,
    ) -> Vec<MethodType> {
        self.module
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Procedure(procedure)
                    if procedure
                        .heading
                        .receiver
                        .as_ref()
                        .is_some_and(|receiver| receiver.ty == type_name) =>
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

    fn record_type_info(&self, type_name: &str) -> Option<(Option<RecordFlavor>, Option<String>)> {
        self.module.declarations.iter().find_map(|declaration| match declaration {
            Declaration::Type(type_decl) if type_decl.name.name == type_name => match &type_decl.ty {
                TypeExpr::Record { flavor, base, .. } => Some((*flavor, base.as_ref().map(|item| item.name.clone()))),
                _ => None,
            },
            _ => None,
        })
    }

    fn find_record_decl(&self, type_name: &str) -> Option<&'a TypeDecl> {
        self.module.declarations.iter().find_map(|declaration| match declaration {
            Declaration::Type(type_decl) if type_decl.name.name == type_name => Some(type_decl),
            _ => None,
        })
    }

    fn find_inherited_method(&self, type_name: &str, method_name: &str) -> Option<&'a ProcedureDecl> {
        let mut current = self.record_type_info(type_name).and_then(|(_, base)| base);
        while let Some(base_name) = current {
            if let Some(method) = self.module.declarations.iter().find_map(|declaration| match declaration {
                Declaration::Procedure(procedure)
                    if procedure.heading.receiver.as_ref().is_some_and(|receiver| receiver.ty == base_name)
                        && procedure.heading.name.name == method_name =>
                {
                    Some(procedure)
                }
                _ => None,
            }) {
                return Some(method);
            }
            current = self.record_type_info(&base_name).and_then(|(_, base)| base);
        }
        None
    }

    fn effective_methods_for_type(&self, type_name: &str) -> Vec<&'a ProcedureDecl> {
        let mut methods = self
            .record_type_info(type_name)
            .and_then(|(_, base)| base)
            .map(|base| self.effective_methods_for_type(&base))
            .unwrap_or_default();

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
        scope_type_names: &HashSet<String>,
    ) -> SemanticType {
        if ident.module.is_none() {
            if let Some(builtin) = builtin_type_by_name(&ident.name) {
                return SemanticType::Builtin(builtin);
            }
            if scope_type_names.contains(&ident.name) {
                return SemanticType::Named {
                    module: None,
                    name: ident.name.clone(),
                    kind: NamedTypeKind::UserDefined,
                };
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
                Statement::Empty { .. } | Statement::Exit { .. } => {}
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
                        if !self.types_are_assignment_compatible(
                            &target_type,
                            &value_type,
                            local_symbols,
                        ) {
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
                        if let Some(guard) = &arm.guard {
                            self.walk_guard(guard, procedure_name, scope_type_names, resolutions);
                            self.validate_with_guard(
                                guard,
                                procedure_name,
                                scope_type_names,
                                local_symbols,
                                diagnostics,
                            );
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
                                if !self.types_are_assignment_compatible(
                                    expected,
                                    &actual,
                                    local_symbols,
                                ) {
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
                        if left_type.as_ref().is_some_and(|ty| is_numeric_type(ty))
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
            None
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
        let (line, column) = designator_position(designator);

        if designator
            .selectors
            .iter()
            .any(|selector| matches!(selector, Selector::Call(_) | Selector::TypeGuard(_) | Selector::AmbiguousParen(_)))
        {
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

                let left_type = self.infer_expr_type(left, local_symbols, scope_type_names);
                let right_type = self.infer_expr_type(right, local_symbols, scope_type_names);
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
                        BinaryOp::Divide => is_numeric_type(&left_type) && is_numeric_type(&right_type),
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
        // Imported types are opaque — trust that the programmer named an extension record.
        let target_is_known_record = self.is_record_type(&target_type, local_symbols)
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

        // Skip the extends check for imported types — we have no definition to verify against.
        let either_imported = matches!(target_type, SemanticType::Named { kind: NamedTypeKind::Imported, .. })
            || matches!(static_record_type, SemanticType::Named { kind: NamedTypeKind::Imported, .. });
        if !either_imported && !self.record_type_extends(&target_type, &static_record_type, local_symbols) {
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
        match subject_type {
            SemanticType::Pointer { target } => {
                if self.is_record_type(target, local_symbols) {
                    Some((**target).clone())
                } else {
                    None
                }
            }
            _ if matches!(subject_symbol.map(|symbol| symbol.kind), Some(SymbolKind::Parameter | SymbolKind::Receiver))
                && (self.is_record_type(subject_type, local_symbols)
                    || matches!(subject_type, SemanticType::Named { kind: NamedTypeKind::Imported, .. })) =>
            {
                Some(subject_type.clone())
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
                        SemanticType::Pointer { target: expected_target },
                        SemanticType::Pointer { target: actual_target },
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
        resolve_named_type_alias(ty, local_symbols, &self.module_symbols, &mut HashSet::new())
            .cloned()
            .unwrap_or_else(|| ty.clone())
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
            None
        };

        if current.is_none() && designator.base.module.is_none() {
            let (line, column) = designator_position(designator);
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                format!("identifier {} is not declared", designator.base.name),
            ));
            return None;
        }

        let mut current = current?;
        for selector in &designator.selectors {
            match self.validate_selector(
                &current,
                selector,
                designator,
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
                None if matches!(base, SemanticType::Record { .. }) => {
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
                    None
                }
                // Imported or cross-module Named types: we can't verify field/method existence
                // without loading the imported module's symbol table.  Suppress the error and
                // return None so sema does not cascade false positives on the remaining selectors.
                None if matches!(
                    base,
                    SemanticType::Named { kind: NamedTypeKind::Imported, .. }
                        | SemanticType::Named { kind: NamedTypeKind::Unresolved, .. }
                ) => None,
                _ => {
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
                match base {
                    SemanticType::Array { element_type, .. } => Some((**element_type).clone()),
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
            Selector::Dereference => match base {
                SemanticType::Pointer { target } => Some((**target).clone()),
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
                if matches!(base, SemanticType::Procedure(_) | SemanticType::BuiltinProc(_)) {
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
                    match base {
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
            Selector::Call(args) => match base {
                SemanticType::Procedure(signature) => {
                    self.validate_call_arguments(
                        signature,
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
                        *proc,
                        args,
                        procedure_name,
                        local_symbols,
                        scope_type_names,
                        diagnostics,
                    );
                    builtin_proc_result_type(
                        *proc,
                        args,
                        local_symbols,
                        &self.module_symbols,
                        scope_type_names,
                    )
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
            designator_position(designator),
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
                if !self.types_are_assignment_compatible(&expected.ty, &actual, local_symbols) {
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
                let (line, column) = designator_position(designator);
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
                SemanticType::Pointer { target } => Some((**target).clone()),
                // Super-call passthrough: v.M^ yields the same procedure type
                SemanticType::Procedure(_) => Some(base.clone()),
                // Imported / unresolved named types may be pointers — don't reject them
                SemanticType::Named { kind: NamedTypeKind::Imported | NamedTypeKind::Unresolved, .. } => None,
                _ => None,
            },
            Selector::TypeGuard(guard) => {
                Some(self.resolve_named_type(guard, scope_type_names))
            }
            Selector::AmbiguousParen(guard) => match base {
                SemanticType::Procedure(signature) => {
                    signature.result_type.as_ref().map(|result| (**result).clone())
                }
                SemanticType::BuiltinProc(proc) => builtin_proc_result_type(
                    *proc,
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
            },
            Selector::Call(_) => match base {
                SemanticType::Procedure(signature) => {
                    signature.result_type.as_ref().map(|result| (**result).clone())
                }
                SemanticType::BuiltinProc(proc) => {
                    builtin_proc_result_type(*proc, &[], &[], &self.module_symbols, scope_type_names)
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

    fn types_are_assignment_compatible(
        &self,
        expected: &SemanticType,
        actual: &SemanticType,
        local_symbols: &[SemanticSymbol],
    ) -> bool {
        let expected = self.resolve_named_type_one_level(expected, local_symbols);
        let actual = self.resolve_named_type_one_level(actual, local_symbols);

        if expected == actual {
            return true;
        }

        if matches!(actual, SemanticType::Nil)
            && matches!(expected, SemanticType::Pointer { .. } | SemanticType::Procedure(_))
        {
            return true;
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
            (SemanticType::Pointer { target: expected_target }, SemanticType::Pointer { target: actual_target }) => {
                self.record_type_extends(actual_target, expected_target, local_symbols)
            }
            (SemanticType::Procedure(expected_sig), SemanticType::Procedure(actual_sig)) => {
                procedure_types_match(expected_sig, actual_sig)
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
        },
        SemanticSymbol {
            name: "FALSE".to_string(),
            kind: SymbolKind::Constant,
            exported: false,
            read_only_export: false,
            declared_type: Some(SemanticType::Builtin(BuiltinType::Boolean)),
            const_value: Some(ConstValue::Boolean(false)),
            simd_shape: None,
        },
        SemanticSymbol {
            name: "INF".to_string(),
            kind: SymbolKind::Constant,
            exported: false,
            read_only_export: false,
            declared_type: Some(SemanticType::Builtin(BuiltinType::Real)),
            const_value: None,
            simd_shape: None,
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
    }));

    symbols
}

fn annotate_simd_shapes(symbols: &mut [SemanticSymbol], outer_symbols: &[SemanticSymbol]) {
    let snapshot = symbols.to_vec();
    for symbol in symbols.iter_mut() {
        symbol.simd_shape = symbol
            .declared_type
            .as_ref()
            .and_then(|ty| infer_simd_shape(ty, &snapshot, outer_symbols, &mut HashSet::new()));
    }
}

fn infer_simd_shape(
    ty: &SemanticType,
    local_symbols: &[SemanticSymbol],
    outer_symbols: &[SemanticSymbol],
    seen_named: &mut HashSet<String>,
) -> Option<SimdShape> {
    match resolve_named_type_alias(ty, local_symbols, outer_symbols, seen_named).unwrap_or(ty) {
        SemanticType::Record {
            base,
            flavor,
            fields,
            methods,
        } if base.is_none() && flavor.is_none() && methods.is_empty() => {
            let (lane_kind, lane_count) = infer_homogeneous_record_lanes(
                fields,
                local_symbols,
                outer_symbols,
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
                resolve_simd_scalar_lane(element_type, local_symbols, outer_symbols, seen_named)
            {
                return Some(make_simd_shape(SimdLayout::ScalarArray, lane_kind, 1));
            }
            match infer_simd_shape(element_type, local_symbols, outer_symbols, seen_named)? {
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
    seen_named: &mut HashSet<String>,
) -> Option<(SimdLaneKind, usize)> {
    let mut lane_kind = None;
    let mut lane_count = 0usize;

    for field in fields {
        let field_lane = resolve_simd_scalar_lane(&field.ty, local_symbols, outer_symbols, seen_named)?;
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
    seen_named: &mut HashSet<String>,
) -> Option<SimdLaneKind> {
    match resolve_named_type_alias(ty, local_symbols, outer_symbols, seen_named).unwrap_or(ty) {
        SemanticType::Builtin(BuiltinType::ShortReal) => Some(SimdLaneKind::Float32),
        SemanticType::Builtin(BuiltinType::Real) => Some(SimdLaneKind::Float64),
        SemanticType::Builtin(BuiltinType::Integer) => Some(SimdLaneKind::Int32),
        SemanticType::Builtin(BuiltinType::LongInt) => Some(SimdLaneKind::Int64),
        _ => None,
    }
}

fn resolve_named_type_alias<'a>(
    ty: &'a SemanticType,
    local_symbols: &'a [SemanticSymbol],
    outer_symbols: &'a [SemanticSymbol],
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
        _ => None,
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
        BuiltinProc::New
        | BuiltinProc::Assert
        | BuiltinProc::Dec
        | BuiltinProc::Excl
        | BuiltinProc::Halt
        | BuiltinProc::Inc
        | BuiltinProc::Incl => None,
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
        SemanticType::Builtin(BuiltinType::ShortInt) => Some(SemanticType::Builtin(BuiltinType::Integer)),
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
        SemanticType::Builtin(BuiltinType::Integer) => Some(SemanticType::Builtin(BuiltinType::ShortInt)),
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
    // including named types that alias pointers/procedures.
    let nil_compatible = |other: &SemanticType| {
        matches!(
            other,
            SemanticType::Pointer { .. }
                | SemanticType::Procedure(_)
                | SemanticType::Named { .. }
        )
    };
    left == right
        || (is_numeric_type(left) && is_numeric_type(right))
        || (is_character_like_type(left) && is_character_like_type(right))
        || (is_string_type(left) && is_string_type(right))
        || (matches!(left, SemanticType::Nil) && nil_compatible(right))
        || (matches!(right, SemanticType::Nil) && nil_compatible(left))
}

fn are_ordered_relation_compatible(left: &SemanticType, right: &SemanticType) -> bool {
    (is_numeric_type(left) && is_numeric_type(right))
        || (is_character_like_type(left) && is_character_like_type(right))
        || (is_string_type(left) && is_string_type(right))
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
        SemanticType::Builtin(BuiltinType::Integer) => 2,
        SemanticType::Builtin(BuiltinType::LongInt) => 3,
        SemanticType::Builtin(BuiltinType::ShortReal) => 4,
        SemanticType::Builtin(BuiltinType::Real) => 5,
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
            let inner = evaluate_const_expr(expr, local_symbols, module_symbols)?;
            match (op, inner) {
                (newcp_parser::UnaryOp::Plus,  ConstValue::Integer(v)) => Some(ConstValue::Integer(v)),
                (newcp_parser::UnaryOp::Minus, ConstValue::Integer(v)) => v.checked_neg().map(ConstValue::Integer),
                (newcp_parser::UnaryOp::Plus,  ConstValue::Real(v))    => Some(ConstValue::Real(v)),
                (newcp_parser::UnaryOp::Minus, ConstValue::Real(v))    => Some(ConstValue::Real(-v)),
                (newcp_parser::UnaryOp::Not,   ConstValue::Boolean(v)) => Some(ConstValue::Boolean(!v)),
                _ => None,
            }
        }
        Expr::Binary { left, op, right, .. } => {
            let lv = evaluate_const_expr(left, local_symbols, module_symbols)?;
            let rv = evaluate_const_expr(right, local_symbols, module_symbols)?;
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

fn filter_symbols(symbols: &[SemanticSymbol], kind: SymbolKind, exported: bool) -> Vec<&str> {
    symbols
        .iter()
        .filter(|symbol| symbol.kind == kind && symbol.exported == exported)
        .map(|symbol| symbol.name.as_str())
        .collect()
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
        } => format!("ARRAY {} OF {}", lengths.join(", "), render_semantic_type(element_type)),
        SemanticType::Record { flavor, base, fields, methods } => {
            let flavor = flavor
                .map(|item| match item {
                    RecordFlavor::Abstract => "ABSTRACT ",
                    RecordFlavor::Extensible => "EXTENSIBLE ",
                    RecordFlavor::Limited => "LIMITED ",
                })
                .unwrap_or("");
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
            format!("{}RECORD {}{} END", flavor, base, fields)
        }
        SemanticType::Pointer { target } => format!("POINTER TO {}", render_semantic_type(target)),
        SemanticType::Procedure(signature) => render_procedure_type(signature),
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
                        SemanticType::Pointer { target } => {
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
                        SemanticType::Builtin(BuiltinType::Char)
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
                            "ORD argument 1 must be CHAR, SHORTCHAR, or SET, found {}",
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

fn render_string_list(items: &[String]) -> String {
    if items.is_empty() {
        "<none>".to_string()
    } else {
        items.join(", ")
    }
}

fn render_str_list(items: &[&str]) -> String {
    if items.is_empty() {
        "<none>".to_string()
    } else {
        items.join(", ")
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
        assert!(
            sema.diagnostics
                .iter()
                .any(|item| item.message.contains("invalid operands for /: SET and SET")),
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
            Some(SemanticType::Builtin(BuiltinType::Integer)),
            "integer constant stores INTEGER declared_type"
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
