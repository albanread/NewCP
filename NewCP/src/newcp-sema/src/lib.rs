use std::collections::HashSet;
use std::path::Path;

use newcp_parser::{
    read_module_ast, BinaryOp, Declaration, Designator, Expr, FPSection, FieldDecl,
    FormalParameters, Guard, MethodFlavor, ModuleAst, ParamMode, ProcedureBody, ProcedureDecl,
    QualIdent, RecordFlavor, Selector, Statement, TypeExpr,
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
            Self::ShortChar => "SHORTCHAR",
            Self::ShortInt => "SHORTINT",
            Self::ShortReal => "SHORTREAL",
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
pub enum SemanticType {
    Builtin(BuiltinType),
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
    },
    Pointer {
        target: Box<SemanticType>,
    },
    Procedure(ProcedureType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub exported: bool,
    pub declared_type: Option<SemanticType>,
    pub const_value: Option<i128>,
    pub simd_shape: Option<SimdShape>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProcedure {
    pub name: String,
    pub exported: bool,
    pub signature: ProcedureType,
    pub local_symbols: Vec<SemanticSymbol>,
    pub selector_resolutions: Vec<SelectorResolution>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
                declared_type: None,
                const_value: None,
                simd_shape: None,
            });
        }

        for declaration in &self.module.declarations {
            match declaration {
                Declaration::Const(item) => {
                    Self::record_identdef_duplicate(&mut scope_names, &item.name, None, "duplicate module-scope declaration", &mut self.diagnostics);
                    let const_value = evaluate_const_integer(&item.value, &[], &self.module_symbols);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Constant,
                        exported: item.name.export.is_some(),
                        declared_type: None,
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
                        declared_type: Some(self.resolve_type_expr(&item.ty, &self.module_type_names)),
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
                            declared_type: Some(declared_type.clone()),
                            const_value: None,
                            simd_shape: None,
                        });
                    }
                }
                Declaration::Procedure(item) => {
                    Self::record_identdef_duplicate(
                        &mut scope_names,
                        &item.heading.name,
                        None,
                        "duplicate module-scope declaration",
                        &mut self.diagnostics,
                    );
                    Self::validate_heading_types(&item.heading, &self.module_type_names, None, &mut self.diagnostics);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: item.heading.name.export.is_some(),
                        declared_type: Some(SemanticType::Procedure(
                            self.resolve_procedure_signature(&self.module_type_names, item),
                        )),
                        const_value: None,
                        simd_shape: None,
                    })
                }
                Declaration::Forward(item) => {
                    Self::record_identdef_duplicate(
                        &mut scope_names,
                        &item.heading.name,
                        None,
                        "duplicate module-scope declaration",
                        &mut self.diagnostics,
                    );
                    Self::validate_heading_types(&item.heading, &self.module_type_names, None, &mut self.diagnostics);
                    self.module_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: item.heading.name.export.is_some(),
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
            let implementation = declarations.iter().find_map(|item| match item {
                Declaration::Procedure(procedure)
                    if procedure.heading.name.name == forward.heading.name.name =>
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
                    let const_value = evaluate_const_integer(&item.value, local_symbols, &self.module_symbols);
                    local_symbols.push(SemanticSymbol {
                        name: item.name.name.clone(),
                        kind: SymbolKind::Constant,
                        exported: false,
                        declared_type: None,
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
                        declared_type: Some(self.resolve_type_expr(&item.ty, scope_type_names)),
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
                            declared_type: Some(declared_type.clone()),
                            const_value: None,
                            simd_shape: None,
                        });
                    }
                }
                Declaration::Procedure(item) => {
                    Self::record_identdef_duplicate(scope_names, &item.heading.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    Self::validate_heading_types(&item.heading, scope_type_names, procedure_name, diagnostics);
                    local_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: false,
                        declared_type: Some(SemanticType::Procedure(self.resolve_procedure_signature(
                            scope_type_names,
                            item,
                        ))),
                        const_value: None,
                        simd_shape: None,
                    })
                }
                Declaration::Forward(item) => {
                    Self::record_identdef_duplicate(scope_names, &item.heading.name, procedure_name, "duplicate procedure-scope declaration", diagnostics);
                    Self::validate_heading_types(&item.heading, scope_type_names, procedure_name, diagnostics);
                    local_symbols.push(SemanticSymbol {
                        name: item.heading.name.name.clone(),
                        kind: SymbolKind::Procedure,
                        exported: false,
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
        match ty {
            TypeExpr::QualIdent { ident, .. } => self.resolve_named_type(ident, scope_type_names),
            TypeExpr::Array {
                lengths,
                element_type,
                ..
            } => SemanticType::Array {
                lengths: lengths.iter().map(render_expr).collect(),
                element_type: Box::new(self.resolve_type_expr(element_type, scope_type_names)),
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
            },
            TypeExpr::Pointer { target, .. } => SemanticType::Pointer {
                target: Box::new(self.resolve_type_expr(target, scope_type_names)),
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
                    .map(|result| Box::new(self.resolve_type_expr(result, scope_type_names))),
                is_new: false,
                flavor: None,
            }),
        }
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
                        if !types_are_assignment_compatible(&target_type, &value_type) {
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
                            Some(0) => {
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
                                if !types_are_assignment_compatible(expected, &actual) {
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

                if !types_are_assignment_compatible(&selector_type, &start_type) {
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
                            if !types_are_assignment_compatible(&selector_type, &end_type) {
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
            Expr::Literal { value, .. } => !matches!(value, newcp_parser::Literal::String(_) | newcp_parser::Literal::Real(_)),
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
                newcp_parser::Literal::Integer(_) => Some(SemanticType::Builtin(BuiltinType::Integer)),
                newcp_parser::Literal::Real(_) => Some(SemanticType::Builtin(BuiltinType::Real)),
                newcp_parser::Literal::Character(_) => Some(SemanticType::Builtin(BuiltinType::Char)),
                newcp_parser::Literal::String(_) => None,
            },
            Expr::Nil { .. } => None,
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
                        infer_numeric_result(left_type.as_ref(), right_type.as_ref())
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
                            Some(SemanticType::Builtin(BuiltinType::Integer))
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
        if !self.is_record_type(&target_type, local_symbols) {
            let (line, column) = position;
            diagnostics.push(make_diagnostic(
                procedure_name,
                line,
                column,
                context.invalid_target_message(&render_qualident(target_ident)),
            ));
            return;
        }

        if !self.record_type_extends(&target_type, &static_record_type, local_symbols) {
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
                && self.is_record_type(subject_type, local_symbols) =>
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
            Selector::Field(name) => match base {
                SemanticType::Record { fields, .. } => fields
                    .iter()
                    .find(|field| field.names.iter().any(|field_name| field_name == name))
                    .map(|field| field.ty.clone())
                    .or_else(|| {
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
                    }),
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
                if self.qualident_denotes_type_name(guard, local_symbols) {
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
                if !types_are_assignment_compatible(&expected.ty, &actual) {
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
            Selector::Field(name) => match base {
                SemanticType::Record { fields, .. } => fields
                    .iter()
                    .find(|field| field.names.iter().any(|field_name| field_name == name))
                    .map(|field| field.ty.clone()),
                _ => None,
            },
            Selector::Index(_) => match base {
                SemanticType::Array { element_type, .. } => Some((**element_type).clone()),
                _ => None,
            },
            Selector::Dereference => match base {
                SemanticType::Pointer { target } => Some((**target).clone()),
                _ => None,
            },
            Selector::TypeGuard(guard) | Selector::AmbiguousParen(guard) => {
                Some(self.resolve_named_type(guard, scope_type_names))
            }
            Selector::Call(_) => match base {
                SemanticType::Procedure(signature) => {
                    signature.result_type.as_ref().map(|result| (**result).clone())
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
                    let resolved_type = self.resolve_named_type(qualident, scope_type_names);
                    resolutions.push(SelectorResolution {
                        procedure: procedure_name.map(str::to_string),
                        designator: rendered.clone(),
                        selector: render_qualident(qualident),
                        kind: selector_resolution_kind_for_type(&resolved_type),
                        reason: format!(
                            "ambiguous parenthesized selector resolves as {}",
                            render_semantic_type(&resolved_type)
                        ),
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
    builtin_types()
        .iter()
        .map(|builtin| SemanticSymbol {
            name: builtin.name().to_string(),
            kind: SymbolKind::Type,
            exported: false,
            declared_type: Some(SemanticType::Builtin(*builtin)),
            const_value: None,
            simd_shape: None,
        })
        .collect()
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
        SemanticType::Record { base, flavor, fields } if base.is_none() && flavor.is_none() => {
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
        BuiltinType::ShortChar,
        BuiltinType::ShortInt,
        BuiltinType::ShortReal,
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
        "SHORTCHAR" => Some(BuiltinType::ShortChar),
        "SHORTINT" => Some(BuiltinType::ShortInt),
        "SHORTREAL" => Some(BuiltinType::ShortReal),
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

fn infer_numeric_result(left: Option<&SemanticType>, right: Option<&SemanticType>) -> Option<SemanticType> {
    match (left, right) {
        (Some(left), Some(right)) if is_numeric_type(left) && is_numeric_type(right) => {
            if matches!(left, SemanticType::Builtin(BuiltinType::Real))
                || matches!(right, SemanticType::Builtin(BuiltinType::Real))
            {
                Some(SemanticType::Builtin(BuiltinType::Real))
            } else {
                Some(SemanticType::Builtin(BuiltinType::Integer))
            }
        }
        _ => None,
    }
}

fn types_are_assignment_compatible(expected: &SemanticType, actual: &SemanticType) -> bool {
    if expected == actual {
        return true;
    }

    is_numeric_type(expected) && is_numeric_type(actual)
}

fn are_relation_compatible(left: &SemanticType, right: &SemanticType) -> bool {
    left == right
        || (is_numeric_type(left) && is_numeric_type(right))
        || (is_character_like_type(left) && is_character_like_type(right))
}

fn are_ordered_relation_compatible(left: &SemanticType, right: &SemanticType) -> bool {
    (is_numeric_type(left) && is_numeric_type(right))
        || (is_character_like_type(left) && is_character_like_type(right))
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

fn evaluate_const_integer(
    expr: &Expr,
    local_symbols: &[SemanticSymbol],
    module_symbols: &[SemanticSymbol],
) -> Option<i128> {
    match expr {
        Expr::Literal { value, .. } => match value {
            newcp_parser::Literal::Integer(value) => parse_component_pascal_integer(value),
            _ => None,
        },
        Expr::Unary { op, expr, .. } => {
            let inner = evaluate_const_integer(expr, local_symbols, module_symbols)?;
            match op {
                newcp_parser::UnaryOp::Plus => Some(inner),
                newcp_parser::UnaryOp::Minus => inner.checked_neg(),
                _ => None,
            }
        }
        Expr::Binary { left, op, right, .. } => {
            let lv = evaluate_const_integer(left, local_symbols, module_symbols)?;
            let rv = evaluate_const_integer(right, local_symbols, module_symbols)?;
            match op {
                BinaryOp::Add => lv.checked_add(rv),
                BinaryOp::Subtract => lv.checked_sub(rv),
                BinaryOp::Multiply => lv.checked_mul(rv),
                BinaryOp::Div => {
                    if rv == 0 { return None; }
                    lv.checked_div(rv)
                }
                BinaryOp::Mod => {
                    if rv == 0 { return None; }
                    lv.checked_rem(rv)
                }
                _ => None,
            }
        }
        Expr::Designator(d) if d.selectors.is_empty() && d.base.module.is_none() => {
            local_symbols.iter().rev()
                .chain(module_symbols.iter().rev())
                .find(|s| s.name == d.base.name && s.kind == SymbolKind::Constant)
                .and_then(|s| s.const_value)
        }
        _ => None,
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

fn render_module_dump(path: &Path, module: &SemanticModule) -> String {
    let exported_constants = filter_symbols(&module.symbols, SymbolKind::Constant, true);
    let exported_types = filter_symbols(&module.symbols, SymbolKind::Type, true);
    let exported_variables = filter_symbols(&module.symbols, SymbolKind::Variable, true);
    let exported_procedures = module
        .procedures
        .iter()
        .filter(|procedure| procedure.exported)
        .map(|procedure| {
            format!(
                "{}:{}",
                procedure.name,
                if procedure.signature.parameters.is_empty() && procedure.signature.result_type.is_none() {
                    "Command"
                } else {
                    "Procedure"
                }
            )
        })
        .collect::<Vec<_>>();
    let local_procedures = module
        .procedures
        .iter()
        .filter(|procedure| !procedure.exported)
        .map(|procedure| procedure.name.as_str())
        .collect::<Vec<_>>();
    let visible_types = module
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Type)
        .map(|symbol| match &symbol.declared_type {
            Some(ty) => format!("{}={}", symbol.name, render_semantic_type(ty)),
            None => symbol.name.clone(),
        })
        .collect::<Vec<_>>();
    let selector_resolutions = if module.selector_resolutions.is_empty() {
        "<none>".to_string()
    } else {
        module
            .selector_resolutions
            .iter()
            .map(|item| {
                let procedure = item.procedure.as_deref().unwrap_or("<module>");
                format!(
                    "{}:{}:{} [{}]",
                    procedure,
                    item.designator,
                    item.selector,
                    item.kind.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let typed_variables = module
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == SymbolKind::Variable)
        .filter_map(|symbol| {
            symbol
                .declared_type
                .as_ref()
                .map(|ty| format!("{}:{}", symbol.name, render_semantic_type(ty)))
        })
        .collect::<Vec<_>>();
    let simd_candidates = module
        .symbols
        .iter()
        .filter_map(|symbol| {
            symbol
                .simd_shape
                .as_ref()
                .map(|shape| format!("{}:{}", symbol.name, render_simd_shape(shape)))
        })
        .collect::<Vec<_>>();
    let diagnostics = if module.diagnostics.is_empty() {
        "<none>".to_string()
    } else {
        module
            .diagnostics
            .iter()
            .map(|item| {
                let procedure = item.procedure.as_deref().unwrap_or("<module>");
                format!("{}@{}:{}: {}", procedure, item.line, item.column, item.message)
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        concat!(
            "newcp-sema module dump\n",
            "input: {}\n",
            "module: {}\n",
            "imports: {}\n",
            "exported-constants: {}\n",
            "exported-types: {}\n",
            "exported-variables: {}\n",
            "exported-procedures: {}\n",
            "local-procedures: {}\n",
            "visible-types: {}\n",
            "typed-variables: {}\n",
            "simd-candidates: {}\n",
            "selector-resolutions: {}\n",
            "diagnostics: {}\n",
            "interface-symbol-count: {}\n",
            "procedure-count: {}"
        ),
        path.display(),
        module.name,
        render_string_list(&module.imports),
        render_str_list(&exported_constants),
        render_str_list(&exported_types),
        render_str_list(&exported_variables),
        render_string_list(&exported_procedures),
        render_str_list(&local_procedures),
        render_string_list(&visible_types),
        render_string_list(&typed_variables),
        render_string_list(&simd_candidates),
        selector_resolutions,
        diagnostics,
        module.symbols.iter().filter(|symbol| symbol.exported).count(),
        module.procedures.len()
    )
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
        SemanticType::Named { module, name, kind } => match kind {
            NamedTypeKind::UserDefined => format!("type:{}", render_optional_module_name(module, name)),
            NamedTypeKind::Imported => format!("imported:{}", render_optional_module_name(module, name)),
            NamedTypeKind::Unresolved => format!("unresolved:{}", render_optional_module_name(module, name)),
        },
        SemanticType::Array {
            lengths,
            element_type,
        } => format!("ARRAY {} OF {}", lengths.join(", "), render_semantic_type(element_type)),
        SemanticType::Record { flavor, base, fields } => {
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
            let fields = if fields.is_empty() {
                String::new()
            } else {
                format!(
                    " {}",
                    fields
                        .iter()
                        .map(|field| format!("{}: {}", field.names.join(", "), render_semantic_type(&field.ty)))
                        .collect::<Vec<_>>()
                        .join("; ")
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

        assert!(dump.contains("module: Demo"));
        assert!(dump.contains("exported-constants: Version"));
        assert!(dump.contains("exported-variables: Current"));
        assert!(dump.contains("exported-procedures: Run:Command"));
        assert!(dump.contains("typed-variables: Current:INTEGER"));
        assert!(dump.contains("simd-candidates: <none>"));
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

        assert!(dump.contains("Pair:packed-record:f64x2:16B"));
        assert!(dump.contains("pairs:array-of-struct:f64x2:16B"));
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
}
