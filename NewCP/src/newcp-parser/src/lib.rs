use std::path::Path;

use newcp_lexer::{lex_source, SourcePosition, SourceSpan, Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceExportKind {
    Constant,
    Type,
    Variable,
    Procedure,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceExport {
    pub name: String,
    pub kind: SourceExportKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProcedure {
    pub name: String,
    pub exported: bool,
    pub has_parameters: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModuleSpec {
    pub name: String,
    pub imports: Vec<String>,
    pub exports: Vec<SourceExport>,
    pub procedures: Vec<SourceProcedure>,
    /// True when the source file declares `DEFINITION MODULE Foo;`.
    /// A definition module provides type signatures for the type-checker and
    /// IR lowerer but is never compiled into JIT code — the real implementation
    /// is provided by a Rust-native module registered with the same name.
    pub is_definition: bool,
}

impl SourceModuleSpec {
    pub fn command_exports(&self) -> Vec<String> {
        self.exports
            .iter()
            .filter(|export| export.kind == SourceExportKind::Command)
            .map(|export| export.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleAst {
    pub span: SourceSpan,
    pub name: String,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
    pub body: Option<Vec<Statement>>,
    pub close: Option<Vec<Statement>>,
    /// True when the file opened with `DEFINITION MODULE`.
    pub is_definition: bool,
}

impl ModuleAst {
    pub fn to_source_module_spec(&self) -> SourceModuleSpec {
        let mut exports = Vec::new();
        let mut procedures = Vec::new();

        for declaration in &self.declarations {
            match declaration {
                Declaration::Const(const_decl) => {
                    if const_decl.name.export.is_some() {
                        exports.push(SourceExport {
                            name: const_decl.name.name.clone(),
                            kind: SourceExportKind::Constant,
                        });
                    }
                }
                Declaration::Type(type_decl) => {
                    if type_decl.name.export.is_some() {
                        exports.push(SourceExport {
                            name: type_decl.name.name.clone(),
                            kind: SourceExportKind::Type,
                        });
                    }
                }
                Declaration::Var(var_decl) => {
                    for name in &var_decl.names {
                        if name.export.is_some() {
                            exports.push(SourceExport {
                                name: name.name.clone(),
                                kind: SourceExportKind::Variable,
                            });
                        }
                    }
                }
                Declaration::Procedure(procedure) => {
                    let exported = procedure.heading.name.export.is_some();
                    let has_parameters = procedure.heading.formal_parameters.is_some();
                    procedures.push(SourceProcedure {
                        name: procedure.heading.name.name.clone(),
                        exported,
                        has_parameters,
                    });
                    if exported {
                        exports.push(SourceExport {
                            name: procedure.heading.name.name.clone(),
                            kind: if has_parameters {
                                SourceExportKind::Procedure
                            } else {
                                SourceExportKind::Command
                            },
                        });
                    }
                }
                Declaration::Forward(_) => {}
            }
        }

        SourceModuleSpec {
            name: self.name.clone(),
            imports: self.imports.iter().map(|item| item.name.clone()).collect(),
            exports,
            procedures,
            is_definition: self.is_definition,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub span: SourceSpan,
    pub alias: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SysFlag {
    Named(String),
    Numeric(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportMark {
    Exported,
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentDef {
    pub span: SourceSpan,
    pub name: String,
    pub export: Option<ExportMark>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualIdent {
    pub span: SourceSpan,
    pub module: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    Const(ConstDecl),
    Type(TypeDecl),
    Var(VarDecl),
    Procedure(ProcedureDecl),
    Forward(ForwardDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstDecl {
    pub span: SourceSpan,
    pub name: IdentDef,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDecl {
    pub span: SourceSpan,
    pub name: IdentDef,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDecl {
    pub span: SourceSpan,
    pub names: Vec<IdentDef>,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureDecl {
    pub span: SourceSpan,
    pub heading: ProcedureHeading,
    pub body: Option<ProcedureBody>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardDecl {
    pub span: SourceSpan,
    pub heading: ProcedureHeading,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureHeading {
    pub span: SourceSpan,
    pub receiver: Option<Receiver>,
    pub name: IdentDef,
    pub formal_parameters: Option<FormalParameters>,
    pub sys_flag: Option<SysFlag>,
    pub attributes: MethodAttributes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureBody {
    pub span: SourceSpan,
    pub declarations: Vec<Declaration>,
    pub body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalParameters {
    pub span: SourceSpan,
    pub sections: Vec<FPSection>,
    pub result_type: Option<Box<TypeExpr>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    Var,
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FPSection {
    pub span: SourceSpan,
    pub mode: Option<ParamMode>,
    pub names: Vec<String>,
    pub sys_flag: Option<SysFlag>,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receiver {
    pub span: SourceSpan,
    pub mode: Option<ParamMode>,
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodFlavor {
    Abstract,
    Empty,
    Extensible,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MethodAttributes {
    pub is_new: bool,
    pub flavor: Option<MethodFlavor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordFlavor {
    Abstract,
    Extensible,
    Limited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    QualIdent {
        span: SourceSpan,
        ident: QualIdent,
    },
    Array {
        span: SourceSpan,
        sys_flag: Option<SysFlag>,
        lengths: Vec<Expr>,
        element_type: Box<TypeExpr>,
    },
    Record {
        span: SourceSpan,
        flavor: Option<RecordFlavor>,
        sys_flag: Option<SysFlag>,
        base: Option<QualIdent>,
        fields: Vec<FieldDecl>,
    },
    Pointer {
        span: SourceSpan,
        sys_flag: Option<SysFlag>,
        target: Box<TypeExpr>,
    },
    Procedure {
        span: SourceSpan,
        sys_flag: Option<SysFlag>,
        formal_parameters: Option<Box<FormalParameters>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
    pub span: SourceSpan,
    pub names: Vec<IdentDef>,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Empty { span: SourceSpan },
    Assignment { span: SourceSpan, target: Designator, value: Expr },
    ProcedureCall { span: SourceSpan, designator: Designator },
    If {
        span: SourceSpan,
        branches: Vec<IfBranch>,
        else_branch: Option<Vec<Statement>>,
    },
    Case {
        span: SourceSpan,
        expr: Expr,
        arms: Vec<CaseArm>,
        else_branch: Option<Vec<Statement>>,
    },
    While { span: SourceSpan, condition: Expr, body: Vec<Statement> },
    Repeat { span: SourceSpan, body: Vec<Statement>, until: Expr },
    For {
        span: SourceSpan,
        variable: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Vec<Statement>,
    },
    Loop { span: SourceSpan, body: Vec<Statement> },
    With {
        span: SourceSpan,
        arms: Vec<WithArm>,
        else_branch: Option<Vec<Statement>>,
    },
    Exit { span: SourceSpan },
    Return { span: SourceSpan, expr: Option<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfBranch {
    pub span: SourceSpan,
    pub condition: Expr,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseArm {
    pub span: SourceSpan,
    pub labels: Vec<CaseLabel>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseLabel {
    pub span: SourceSpan,
    pub start: Expr,
    pub end: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithArm {
    pub span: SourceSpan,
    pub guard: Option<Guard>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guard {
    pub span: SourceSpan,
    pub variable: QualIdent,
    pub ty: QualIdent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal { span: SourceSpan, value: Literal },
    Nil { span: SourceSpan },
    Designator(Designator),
    Set { span: SourceSpan, elements: Vec<SetElement> },
    Unary { span: SourceSpan, op: UnaryOp, expr: Box<Expr> },
    Binary {
        span: SourceSpan,
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Integer(String),
    Real(String),
    Character(String),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Or,
    Multiply,
    Divide,
    Div,
    Mod,
    And,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    In,
    Is,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetElement {
    pub span: SourceSpan,
    pub start: Expr,
    pub end: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Designator {
    pub span: SourceSpan,
    pub base: QualIdent,
    pub selectors: Vec<Selector>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Field(String),
    Index(Vec<Expr>),
    Dereference,
    TypeGuard(QualIdent),
    Call(Vec<Expr>),
    AmbiguousParen(QualIdent),
    StringDereference,
}

pub fn dump_ast(path: &Path) -> String {
    match read_module_ast(path) {
        Ok(module) => render_module_ast(path, &module),
        Err(error) => format!(
            "newcp-parser source error\ninput: {}\nerror: {}",
            path.display(),
            error
        ),
    }
}

pub fn read_module_ast(path: &Path) -> Result<ModuleAst, String> {
    let source_text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_module_ast(&source_text)
}

pub fn read_source_module(path: &Path) -> Result<SourceModuleSpec, String> {
    Ok(read_module_ast(path)?.to_source_module_spec())
}

pub fn parse_module_ast(source_text: &str) -> Result<ModuleAst, String> {
    let tokens = lex_source(source_text).map_err(|error| error.render())?;
    Parser::new(tokens).parse_module()
}

pub fn parse_source_module(source_text: &str) -> Result<SourceModuleSpec, String> {
    Ok(parse_module_ast(source_text)?.to_source_module_spec())
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    is_definition: bool,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0, is_definition: false }
    }

    fn parse_module(&mut self) -> Result<ModuleAst, String> {
        let start = self.current_start();
        // Accept both:  "MODULE Foo;"  and  "DEFINITION MODULE Foo;"
        let is_definition = if self.match_keyword("DEFINITION") {
            true
        } else {
            false
        };
        self.is_definition = is_definition;
        self.expect_keyword("MODULE")?;
        let name = self.expect_identifier()?;
        self.expect_symbol(";")?;

        let imports = if self.match_keyword("IMPORT") {
            self.parse_import_list()?
        } else {
            Vec::new()
        };

        let declarations = self.parse_declaration_sequence()?;
        let body = if self.match_keyword("BEGIN") {
            Some(self.parse_statement_sequence(&["CLOSE", "END"])? )
        } else {
            None
        };
        let close = if self.match_keyword("CLOSE") {
            Some(self.parse_statement_sequence(&["END"])? )
        } else {
            None
        };

        self.expect_keyword("END")?;
        let closing_name = self.expect_identifier()?;
        if closing_name != name {
            return Err(self.error_current(&format!(
                "module ends with {}, expected {}",
                closing_name, name
            )));
        }
        self.expect_symbol(".")?;

        Ok(ModuleAst {
            span: self.span_from(start),
            name,
            imports,
            declarations,
            body,
            close,
            is_definition,
        })
    }

    fn parse_import_list(&mut self) -> Result<Vec<Import>, String> {
        let mut imports = Vec::new();
        loop {
            let start = self.current_start();
            let first = self.expect_identifier()?;
            let (alias, name) = if self.match_symbol(":=") {
                (Some(first), self.expect_identifier()?)
            } else {
                (None, first)
            };
            imports.push(Import {
                span: self.span_from(start),
                alias,
                name,
            });

            if self.match_symbol(",") {
                continue;
            }
            self.expect_symbol(";")?;
            return Ok(imports);
        }
    }

    fn parse_declaration_sequence(&mut self) -> Result<Vec<Declaration>, String> {
        let mut declarations = Vec::new();

        loop {
            if self.match_keyword("CONST") {
                while self.check_identifier() {
                    declarations.push(Declaration::Const(self.parse_const_declaration()?));
                    self.expect_symbol(";")?;
                }
                continue;
            }

            if self.match_keyword("TYPE") {
                while self.check_identifier() {
                    declarations.push(Declaration::Type(self.parse_type_declaration()?));
                    self.expect_symbol(";")?;
                }
                continue;
            }

            if self.match_keyword("VAR") {
                while self.check_identifier() {
                    declarations.push(Declaration::Var(self.parse_var_declaration()?));
                    self.expect_symbol(";")?;
                }
                continue;
            }

            break;
        }

        while self.check_keyword("PROCEDURE") {
            self.expect_keyword("PROCEDURE")?;
            if self.match_symbol("^") {
                let forward_start = self.current_start();
                let heading = self.parse_procedure_heading()?;
                declarations.push(Declaration::Forward(ForwardDecl {
                    span: self.span_from(forward_start),
                    heading,
                }));
            } else {
                let proc = self.parse_procedure_declaration()?;
                let has_body = proc.body.is_some();
                declarations.push(Declaration::Procedure(proc));
                // Definition modules use bodyless headings; no ";" follows them.
                if has_body || !self.is_definition {
                    self.expect_symbol(";")?;
                } else {
                    // Optionally consume a ";" if present (allows both styles).
                    self.match_symbol(";");
                }
            }
        }

        Ok(declarations)
    }

    fn parse_const_declaration(&mut self) -> Result<ConstDecl, String> {
        let start = self.current_start();
        let name = self.parse_ident_def()?;
        self.expect_symbol("=")?;
        let value = self.parse_expr()?;
        Ok(ConstDecl {
            span: self.span_from(start),
            name,
            value,
        })
    }

    fn parse_type_declaration(&mut self) -> Result<TypeDecl, String> {
        let start = self.current_start();
        let name = self.parse_ident_def()?;
        self.expect_symbol("=")?;
        let ty = self.parse_type_expr()?;
        Ok(TypeDecl {
            span: self.span_from(start),
            name,
            ty,
        })
    }

    fn parse_var_declaration(&mut self) -> Result<VarDecl, String> {
        let start = self.current_start();
        let names = self.parse_ident_list_defs()?;
        self.expect_symbol(":")?;
        let ty = self.parse_type_expr()?;
        Ok(VarDecl {
            span: self.span_from(start),
            names,
            ty,
        })
    }

    fn parse_procedure_declaration(&mut self) -> Result<ProcedureDecl, String> {
        let start = self.current_start();
        let heading = self.parse_procedure_heading()?;
        let body = if matches!(
            heading.attributes.flavor,
            Some(MethodFlavor::Abstract | MethodFlavor::Empty)
        ) {
            None
        } else if !self.is_definition && self.match_symbol(";") {
            let body_start = self.previous_end();
            let declarations = self.parse_declaration_sequence()?;
            let body = if self.match_keyword("BEGIN") {
                Some(self.parse_statement_sequence(&["END"])? )
            } else {
                None
            };

            self.expect_keyword("END")?;
            let closing_name = self.expect_identifier()?;
            if closing_name != heading.name.name {
                return Err(self.error_current(&format!(
                    "procedure ends with {}, expected {}",
                    closing_name, heading.name.name
                )));
            }

            Some(ProcedureBody {
                span: SourceSpan {
                    start: body_start,
                    end: self.previous_end(),
                },
                declarations,
                body,
            })
        } else {
            // In a definition module, consume optional trailing ";" after the heading.
            if self.is_definition { self.match_symbol(";"); }
            None
        };

        Ok(ProcedureDecl {
            span: self.span_from(start),
            heading,
            body,
        })
    }

    fn parse_procedure_heading(&mut self) -> Result<ProcedureHeading, String> {
        let start = self.current_start();
        let sys_flag = self.parse_optional_sys_flag()?;
        let receiver = if self.check_symbol("(") && self.lookahead_is_receiver() {
            Some(self.parse_receiver()?)
        } else {
            None
        };
        let name = self.parse_ident_def()?;
        let formal_parameters = if self.check_symbol("(") {
            Some(self.parse_formal_parameters()?)
        } else {
            None
        };
        let attributes = self.parse_method_attributes()?;

        Ok(ProcedureHeading {
            span: self.span_from(start),
            receiver,
            name,
            formal_parameters,
            sys_flag,
            attributes,
        })
    }

    fn parse_receiver(&mut self) -> Result<Receiver, String> {
        let start = self.current_start();
        self.expect_symbol("(")?;
        let mode = if self.match_keyword("VAR") {
            Some(ParamMode::Var)
        } else if self.match_keyword("IN") {
            Some(ParamMode::In)
        } else {
            None
        };
        let name = self.expect_identifier()?;
        self.expect_symbol(":")?;
        let ty = self.expect_identifier()?;
        self.expect_symbol(")")?;
        Ok(Receiver {
            span: self.span_from(start),
            mode,
            name,
            ty,
        })
    }

    fn parse_method_attributes(&mut self) -> Result<MethodAttributes, String> {
        let mut attributes = MethodAttributes::default();
        if self.match_symbol(",") {
            if self.match_identifier_text("NEW") {
                attributes.is_new = true;
                if self.match_symbol(",") {
                    attributes.flavor = Some(self.parse_method_flavor()?);
                }
            } else {
                attributes.flavor = Some(self.parse_method_flavor()?);
            }
        }
        Ok(attributes)
    }

    fn parse_method_flavor(&mut self) -> Result<MethodFlavor, String> {
        if self.match_keyword("ABSTRACT") {
            Ok(MethodFlavor::Abstract)
        } else if self.match_keyword("EMPTY") {
            Ok(MethodFlavor::Empty)
        } else if self.match_keyword("EXTENSIBLE") {
            Ok(MethodFlavor::Extensible)
        } else {
            Err(self.error_current("expected method attribute"))
        }
    }

    fn parse_formal_parameters(&mut self) -> Result<FormalParameters, String> {
        let start = self.current_start();
        self.expect_symbol("(")?;
        let mut sections = Vec::new();
        if !self.check_symbol(")") {
            loop {
                sections.push(self.parse_fp_section()?);
                if self.match_symbol(";") {
                    continue;
                }
                break;
            }
        }
        self.expect_symbol(")")?;
        let result_type = if self.match_symbol(":") {
            Some(Box::new(self.parse_type_expr()?))
        } else {
            None
        };
        Ok(FormalParameters {
            span: self.span_from(start),
            sections,
            result_type,
        })
    }

    fn parse_fp_section(&mut self) -> Result<FPSection, String> {
        let start = self.current_start();
        let mode = if self.match_keyword("VAR") {
            Some(ParamMode::Var)
        } else if self.match_keyword("IN") {
            Some(ParamMode::In)
        } else if self.match_keyword("OUT") {
            Some(ParamMode::Out)
        } else {
            None
        };
        let sys_flag = self.parse_optional_sys_flag()?;
        let mut names = vec![self.expect_identifier()?];
        while self.match_symbol(",") {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(":")?;
        let ty = self.parse_type_expr()?;
        Ok(FPSection {
            span: self.span_from(start),
            mode,
            names,
            sys_flag,
            ty,
        })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, String> {
        let start = self.current_start();
        if self.match_keyword("ARRAY") {
            let sys_flag = self.parse_optional_sys_flag()?;
            let mut lengths = Vec::new();
            if !self.check_keyword("OF") {
                lengths.push(self.parse_expr()?);
                while self.match_symbol(",") {
                    lengths.push(self.parse_expr()?);
                }
            }
            self.expect_keyword("OF")?;
            let element_type = Box::new(self.parse_type_expr()?);
            return Ok(TypeExpr::Array {
                span: self.span_from(start),
                sys_flag,
                lengths,
                element_type,
            });
        }

        if self.match_keyword("POINTER") {
            let sys_flag = self.parse_optional_sys_flag()?;
            self.expect_keyword("TO")?;
            return Ok(TypeExpr::Pointer {
                span: self.span_from(start),
                sys_flag,
                target: Box::new(self.parse_type_expr()?),
            });
        }

        if self.match_keyword("PROCEDURE") {
            let sys_flag = self.parse_optional_sys_flag()?;
            let formal_parameters = if self.check_symbol("(") {
                Some(Box::new(self.parse_formal_parameters()?))
            } else {
                None
            };
            return Ok(TypeExpr::Procedure {
                span: self.span_from(start),
                sys_flag,
                formal_parameters,
            });
        }

        let flavor = if self.match_keyword("ABSTRACT") {
            Some(RecordFlavor::Abstract)
        } else if self.match_keyword("EXTENSIBLE") {
            Some(RecordFlavor::Extensible)
        } else if self.match_keyword("LIMITED") {
            Some(RecordFlavor::Limited)
        } else {
            None
        };

        if flavor.is_some() || self.check_keyword("RECORD") {
            self.expect_keyword("RECORD")?;
            let sys_flag = self.parse_optional_sys_flag()?;
            let base = if self.match_symbol("(") {
                let base = self.parse_qualident()?;
                self.expect_symbol(")")?;
                Some(base)
            } else {
                None
            };

            let mut fields = Vec::new();
            while self.check_identifier() {
                fields.push(self.parse_field_decl()?);
                if !self.match_symbol(";") {
                    break;
                }
            }
            self.expect_keyword("END")?;
            return Ok(TypeExpr::Record {
                span: self.span_from(start),
                flavor,
                sys_flag,
                base,
                fields,
            });
        }

        Ok(TypeExpr::QualIdent {
            span: self.span_from(start),
            ident: self.parse_qualident()?,
        })
    }

    fn parse_field_decl(&mut self) -> Result<FieldDecl, String> {
        let start = self.current_start();
        let names = self.parse_ident_list_defs()?;
        self.expect_symbol(":")?;
        let ty = self.parse_type_expr()?;
        Ok(FieldDecl {
            span: self.span_from(start),
            names,
            ty,
        })
    }

    fn parse_optional_sys_flag(&mut self) -> Result<Option<SysFlag>, String> {
        if !self.match_symbol("[") {
            return Ok(None);
        }

        let flag = if self.check_identifier() {
            SysFlag::Named(self.expect_identifier()?)
        } else if self.check_symbol("-") {
            self.expect_symbol("-")?;
            if self.current().kind != TokenKind::Integer {
                return Err(self.error_current("expected integer system flag"));
            }
            let value = self.current().lexeme.clone();
            self.advance();
            SysFlag::Numeric(-value.parse::<i64>().map_err(|_| self.error_current("invalid integer system flag"))?)
        } else if self.current().kind == TokenKind::Integer {
            let value = self.current().lexeme.clone();
            self.advance();
            SysFlag::Numeric(value.parse::<i64>().map_err(|_| self.error_current("invalid integer system flag"))?)
        } else {
            return Err(self.error_current("expected system flag"));
        };

        self.expect_symbol("]")?;
        Ok(Some(flag))
    }

    fn parse_statement_sequence(&mut self, stop_markers: &[&str]) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.check_kind(TokenKind::Eof)
            && !stop_markers.iter().any(|marker| self.check_stop_marker(marker))
        {
            if self.match_symbol(";") {
                let empty_span = self.previous_token_span();
                statements.push(Statement::Empty { span: empty_span });
                continue;
            }

            statements.push(self.parse_statement()?);
            if self.match_symbol(";") {
                continue;
            }
        }
        Ok(statements)
    }

    fn check_stop_marker(&self, marker: &str) -> bool {
        self.check_keyword(marker) || self.check_symbol(marker)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let statement_start = self.current_start();
        if self.match_keyword("IF") {
            return self.parse_if_statement(statement_start);
        }
        if self.match_keyword("CASE") {
            return self.parse_case_statement(statement_start);
        }
        if self.match_keyword("WHILE") {
            let condition = self.parse_expr()?;
            self.expect_keyword("DO")?;
            let body = self.parse_statement_sequence(&["END"])?;
            self.expect_keyword("END")?;
            return Ok(Statement::While {
                span: self.span_from(statement_start),
                condition,
                body,
            });
        }
        if self.match_keyword("REPEAT") {
            let body = self.parse_statement_sequence(&["UNTIL"])?;
            self.expect_keyword("UNTIL")?;
            let until = self.parse_expr()?;
            return Ok(Statement::Repeat {
                span: self.span_from(statement_start),
                body,
                until,
            });
        }
        if self.match_keyword("FOR") {
            let variable = self.expect_identifier()?;
            self.expect_symbol(":=")?;
            let start = self.parse_expr()?;
            self.expect_keyword("TO")?;
            let end = self.parse_expr()?;
            let step = if self.match_keyword("BY") {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect_keyword("DO")?;
            let body = self.parse_statement_sequence(&["END"])?;
            self.expect_keyword("END")?;
            return Ok(Statement::For {
                span: self.span_from(statement_start),
                variable,
                start,
                end,
                step,
                body,
            });
        }
        if self.match_keyword("LOOP") {
            let body = self.parse_statement_sequence(&["END"])?;
            self.expect_keyword("END")?;
            return Ok(Statement::Loop {
                span: self.span_from(statement_start),
                body,
            });
        }
        if self.match_keyword("WITH") {
            return self.parse_with_statement(statement_start);
        }
        if self.match_keyword("EXIT") {
            return Ok(Statement::Exit {
                span: self.span_from(statement_start),
            });
        }
        if self.match_keyword("RETURN") {
            let expr = if self.can_start_expr() {
                Some(self.parse_expr()?)
            } else {
                None
            };
            return Ok(Statement::Return {
                span: self.span_from(statement_start),
                expr,
            });
        }

        let designator = self.parse_designator()?;
        if self.match_symbol(":=") {
            let value = self.parse_expr()?;
            Ok(Statement::Assignment {
                span: self.span_from(statement_start),
                target: designator,
                value,
            })
        } else {
            Ok(Statement::ProcedureCall {
                span: self.span_from(statement_start),
                designator,
            })
        }
    }

    fn parse_if_statement(&mut self, start: SourcePosition) -> Result<Statement, String> {
        let mut branches = Vec::new();
        let condition = self.parse_expr()?;
        self.expect_keyword("THEN")?;
        let body = self.parse_statement_sequence(&["ELSIF", "ELSE", "END"])?;
        branches.push(IfBranch {
            span: SourceSpan {
                start: expr_start(&condition),
                end: sequence_end(&body).unwrap_or(expr_end(&condition)),
            },
            condition,
            body,
        });

        while self.match_keyword("ELSIF") {
            let condition = self.parse_expr()?;
            self.expect_keyword("THEN")?;
            let body = self.parse_statement_sequence(&["ELSIF", "ELSE", "END"])?;
            let branch_start = expr_start(&condition);
            let branch_end = sequence_end(&body).unwrap_or(expr_end(&condition));
            branches.push(IfBranch {
                span: SourceSpan {
                    start: branch_start,
                    end: branch_end,
                },
                condition,
                body,
            });
        }

        let else_branch = if self.match_keyword("ELSE") {
            Some(self.parse_statement_sequence(&["END"])? )
        } else {
            None
        };
        self.expect_keyword("END")?;
        Ok(Statement::If {
            span: self.span_from(start),
            branches,
            else_branch,
        })
    }

    fn parse_case_statement(&mut self, start: SourcePosition) -> Result<Statement, String> {
        let expr = self.parse_expr()?;
        self.expect_keyword("OF")?;
        let mut arms = vec![self.parse_case_arm()?];
        while self.match_symbol("|") {
            arms.push(self.parse_case_arm()?);
        }
        let else_branch = if self.match_keyword("ELSE") {
            Some(self.parse_statement_sequence(&["END"])? )
        } else {
            None
        };
        self.expect_keyword("END")?;
        Ok(Statement::Case {
            span: self.span_from(start),
            expr,
            arms,
            else_branch,
        })
    }

    fn parse_case_arm(&mut self) -> Result<CaseArm, String> {
        let start = self.current_start();
        let mut labels = vec![self.parse_case_label()?];
        while self.match_symbol(",") {
            labels.push(self.parse_case_label()?);
        }
        self.expect_symbol(":")?;
        let body = self.parse_statement_sequence(&["|", "ELSE", "END"])?;
        Ok(CaseArm {
            span: self.span_from(start),
            labels,
            body,
        })
    }

    fn parse_case_label(&mut self) -> Result<CaseLabel, String> {
        let span_start = self.current_start();
        let start = self.parse_expr()?;
        let end = if self.match_symbol("..") {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(CaseLabel {
            span: self.span_from(span_start),
            start,
            end,
        })
    }

    fn parse_with_statement(&mut self, start: SourcePosition) -> Result<Statement, String> {
        let mut arms = vec![self.parse_with_arm()?];
        while self.match_symbol("|") {
            arms.push(self.parse_with_arm()?);
        }
        let else_branch = if self.match_keyword("ELSE") {
            Some(self.parse_statement_sequence(&["END"])? )
        } else {
            None
        };
        self.expect_keyword("END")?;
        Ok(Statement::With {
            span: self.span_from(start),
            arms,
            else_branch,
        })
    }

    fn parse_with_arm(&mut self) -> Result<WithArm, String> {
        let start = self.current_start();
        let (guard, body) = if self.can_start_guard() {
            let guard = Some(self.parse_guard()?);
            self.expect_keyword("DO")?;
            let body = self.parse_statement_sequence(&["|", "ELSE", "END"])?;
            (guard, body)
        } else {
            (None, Vec::new())
        };
        Ok(WithArm {
            span: self.span_from(start),
            guard,
            body,
        })
    }

    fn parse_guard(&mut self) -> Result<Guard, String> {
        let start = self.current_start();
        let variable = self.parse_qualident()?;
        self.expect_symbol(":")?;
        let ty = self.parse_qualident()?;
        Ok(Guard {
            span: self.span_from(start),
            variable,
            ty,
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_simple_expr()?;
        if let Some(op) = self.match_relation() {
            let right = self.parse_simple_expr()?;
            expr = Expr::Binary {
                span: SourceSpan {
                    start: expr_start(&expr),
                    end: expr_end(&right),
                },
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_simple_expr(&mut self) -> Result<Expr, String> {
        let start = self.current_start();
        let mut expr = if self.match_symbol("+") {
            Expr::Unary {
                span: self.span_from(start),
                op: UnaryOp::Plus,
                expr: Box::new(self.parse_term()?),
            }
        } else if self.match_symbol("-") {
            Expr::Unary {
                span: self.span_from(start),
                op: UnaryOp::Minus,
                expr: Box::new(self.parse_term()?),
            }
        } else {
            self.parse_term()?
        };

        while let Some(op) = self.match_add_op() {
            let right = self.parse_term()?;
            expr = Expr::Binary {
                span: SourceSpan {
                    start: expr_start(&expr),
                    end: expr_end(&right),
                },
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        while let Some(op) = self.match_mul_op() {
            let right = self.parse_factor()?;
            expr = Expr::Binary {
                span: SourceSpan {
                    start: expr_start(&expr),
                    end: expr_end(&right),
                },
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let start = self.current_start();
        if self.match_symbol("~") {
            return Ok(Expr::Unary {
                span: self.span_from(start),
                op: UnaryOp::Not,
                expr: Box::new(self.parse_factor()?),
            });
        }

        if self.match_symbol("(") {
            let expr = self.parse_expr()?;
            self.expect_symbol(")")?;
            return Ok(expr);
        }

        if self.match_symbol("{") {
            let mut elements = Vec::new();
            if !self.check_symbol("}") {
                elements.push(self.parse_set_element()?);
                while self.match_symbol(",") {
                    elements.push(self.parse_set_element()?);
                }
            }
            self.expect_symbol("}")?;
            return Ok(Expr::Set {
                span: self.span_from(start),
                elements,
            });
        }

        if self.match_keyword("NIL") {
            return Ok(Expr::Nil {
                span: self.span_from(start),
            });
        }

        match self.current().kind {
            TokenKind::Integer => {
                let value = self.current().lexeme.clone();
                self.advance();
                Ok(Expr::Literal {
                    span: self.span_from(start),
                    value: Literal::Integer(value),
                })
            }
            TokenKind::Real => {
                let value = self.current().lexeme.clone();
                self.advance();
                Ok(Expr::Literal {
                    span: self.span_from(start),
                    value: Literal::Real(value),
                })
            }
            TokenKind::Character => {
                let value = self.current().lexeme.clone();
                self.advance();
                Ok(Expr::Literal {
                    span: self.span_from(start),
                    value: Literal::Character(value),
                })
            }
            TokenKind::String => {
                let value = self.current().lexeme.clone();
                self.advance();
                Ok(Expr::Literal {
                    span: self.span_from(start),
                    value: Literal::String(value),
                })
            }
            TokenKind::Identifier => Ok(Expr::Designator(self.parse_designator()?)),
            _ => Err(self.error_current("expected expression factor")),
        }
    }

    fn parse_set_element(&mut self) -> Result<SetElement, String> {
        let span_start = self.current_start();
        let start = self.parse_expr()?;
        let end = if self.match_symbol("..") {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(SetElement {
            span: self.span_from(span_start),
            start,
            end,
        })
    }

    fn parse_designator(&mut self) -> Result<Designator, String> {
        let start = self.current_start();
        let base = self.parse_qualident()?;
        let mut selectors = Vec::new();

        loop {
            if self.match_symbol(".") {
                selectors.push(Selector::Field(self.expect_identifier()?));
                continue;
            }
            if self.match_symbol("[") {
                let indices = self.parse_expr_list()?;
                self.expect_symbol("]")?;
                selectors.push(Selector::Index(indices));
                continue;
            }
            if self.match_symbol("^") {
                selectors.push(Selector::Dereference);
                continue;
            }
            if self.check_symbol("(") {
                selectors.push(self.parse_paren_selector()?);
                continue;
            }
            break;
        }

        if self.match_symbol("$") {
            selectors.push(Selector::StringDereference);
        }

        Ok(Designator {
            span: self.span_from(start),
            base,
            selectors,
        })
    }

    fn parse_paren_selector(&mut self) -> Result<Selector, String> {
        self.expect_symbol("(")?;
        let args = if self.check_symbol(")") {
            Vec::new()
        } else {
            self.parse_expr_list()?
        };
        self.expect_symbol(")")?;
        // `(qualident)` could be either a one-arg call or a type-guard.
        // Emit `AmbiguousParen` for both unqualified (`Foo`) and module-
        // qualified (`Mod.Foo`) single-ident cases so that sema can pick
        // the right interpretation based on whether the name denotes a
        // type. Without the qualified case, expressions like
        // `loc(HostFiles.StdLocator)` were always parsed as a Call and
        // sema refused them with "call selector requires a procedure".
        if let [Expr::Designator(designator)] = args.as_slice() {
            if designator.selectors.is_empty() {
                return Ok(Selector::AmbiguousParen(QualIdent {
                    span: designator.base.span,
                    module: designator.base.module.clone(),
                    name: designator.base.name.clone(),
                }));
            }
        }

        Ok(Selector::Call(args))
    }

    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut items = vec![self.parse_expr()?];
        while self.match_symbol(",") {
            items.push(self.parse_expr()?);
        }
        Ok(items)
    }

    fn parse_ident_list_defs(&mut self) -> Result<Vec<IdentDef>, String> {
        let mut names = vec![self.parse_ident_def()?];
        while self.match_symbol(",") {
            names.push(self.parse_ident_def()?);
        }
        Ok(names)
    }

    fn parse_ident_def(&mut self) -> Result<IdentDef, String> {
        let start = self.current_start();
        let name = self.expect_identifier()?;
        let export = if self.match_symbol("*") {
            Some(ExportMark::Exported)
        } else if self.match_symbol("-") {
            Some(ExportMark::ReadOnly)
        } else {
            None
        };
        Ok(IdentDef {
            span: self.span_from(start),
            name,
            export,
        })
    }

    fn parse_qualident(&mut self) -> Result<QualIdent, String> {
        let start = self.current_start();
        let first = self.expect_identifier()?;
        if self.match_symbol(".") {
            Ok(QualIdent {
                span: self.span_from(start),
                module: Some(first),
                name: self.expect_identifier()?,
            })
        } else {
            Ok(QualIdent {
                span: self.span_from(start),
                module: None,
                name: first,
            })
        }
    }

    fn match_relation(&mut self) -> Option<BinaryOp> {
        if self.match_symbol("=") {
            Some(BinaryOp::Equal)
        } else if self.match_symbol("#") {
            Some(BinaryOp::NotEqual)
        } else if self.match_symbol("<") {
            Some(BinaryOp::Less)
        } else if self.match_symbol("<=") {
            Some(BinaryOp::LessEqual)
        } else if self.match_symbol(">") {
            Some(BinaryOp::Greater)
        } else if self.match_symbol(">=") {
            Some(BinaryOp::GreaterEqual)
        } else if self.match_keyword("IN") {
            Some(BinaryOp::In)
        } else if self.match_keyword("IS") {
            Some(BinaryOp::Is)
        } else {
            None
        }
    }

    fn match_add_op(&mut self) -> Option<BinaryOp> {
        if self.match_symbol("+") {
            Some(BinaryOp::Add)
        } else if self.match_symbol("-") {
            Some(BinaryOp::Subtract)
        } else if self.match_keyword("OR") {
            Some(BinaryOp::Or)
        } else {
            None
        }
    }

    fn match_mul_op(&mut self) -> Option<BinaryOp> {
        if self.match_symbol("*") {
            Some(BinaryOp::Multiply)
        } else if self.match_symbol("/") {
            Some(BinaryOp::Divide)
        } else if self.match_keyword("DIV") {
            Some(BinaryOp::Div)
        } else if self.match_keyword("MOD") {
            Some(BinaryOp::Mod)
        } else if self.match_symbol("&") {
            Some(BinaryOp::And)
        } else {
            None
        }
    }

    fn can_start_expr(&self) -> bool {
        self.check_identifier()
            || matches!(
                self.current().kind,
                TokenKind::Integer | TokenKind::Real | TokenKind::Character | TokenKind::String
            )
            || self.check_keyword("NIL")
            || self.check_symbol("{")
            || self.check_symbol("(")
            || self.check_symbol("~")
            || self.check_symbol("+")
            || self.check_symbol("-")
    }

    fn can_start_guard(&self) -> bool {
        if !self.check_identifier() {
            return false;
        }

        let mut index = self.index + 1;
        if self.check_token(index, TokenKind::Symbol, Some(".")) {
            index += 1;
            if !self.check_token(index, TokenKind::Identifier, None) {
                return false;
            }
            index += 1;
        }

        self.check_token(index, TokenKind::Symbol, Some(":"))
    }

    fn lookahead_is_receiver(&self) -> bool {
        let mut index = self.index;
        if !self.check_token(index, TokenKind::Symbol, Some("(")) {
            return false;
        }
        index += 1;
        if self.check_token(index, TokenKind::Keyword, Some("VAR"))
            || self.check_token(index, TokenKind::Keyword, Some("IN"))
        {
            index += 1;
        }
        self.check_token(index, TokenKind::Identifier, None)
            && self.check_token(index + 1, TokenKind::Symbol, Some(":"))
            && self.check_token(index + 2, TokenKind::Identifier, None)
            && self.check_token(index + 3, TokenKind::Symbol, Some(")"))
    }

    fn check_token(&self, index: usize, kind: TokenKind, lexeme: Option<&str>) -> bool {
        let Some(token) = self.tokens.get(index) else {
            return false;
        };
        token.kind == kind && lexeme.is_none_or(|item| token.lexeme == item)
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        if self.check_identifier() {
            let value = self.current().lexeme.clone();
            self.advance();
            Ok(value)
        } else {
            Err(self.error_current("expected identifier"))
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if self.match_keyword(keyword) {
            Ok(())
        } else {
            Err(self.error_current(&format!("expected keyword {}", keyword)))
        }
    }

    fn expect_symbol(&mut self, symbol: &str) -> Result<(), String> {
        if self.match_symbol(symbol) {
            Ok(())
        } else {
            Err(self.error_current(&format!("expected symbol {}", symbol)))
        }
    }

    fn match_keyword(&mut self, keyword: &str) -> bool {
        if self.check_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_identifier_text(&mut self, text: &str) -> bool {
        if self.check_identifier_text(text) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_symbol(&mut self, symbol: &str) -> bool {
        if self.check_symbol(symbol) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_identifier(&self) -> bool {
        self.current().kind == TokenKind::Identifier
    }

    fn check_identifier_text(&self, text: &str) -> bool {
        self.current().kind == TokenKind::Identifier && self.current().lexeme == text
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        self.current().kind == TokenKind::Keyword && self.current().lexeme == keyword
    }

    fn check_symbol(&self, symbol: &str) -> bool {
        self.current().kind == TokenKind::Symbol && self.current().lexeme == symbol
    }

    fn check_kind(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index.min(self.tokens.len().saturating_sub(1))]
    }

    fn advance(&mut self) {
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
        }
    }

    fn current_start(&self) -> SourcePosition {
        self.current().span.start
    }

    fn previous_end(&self) -> SourcePosition {
        self.tokens
            .get(self.index.saturating_sub(1))
            .map(|token| token.span.end)
            .unwrap_or_else(|| self.current().span.start)
    }

    fn previous_token_span(&self) -> SourceSpan {
        self.tokens
            .get(self.index.saturating_sub(1))
            .map(|token| token.span)
            .unwrap_or(SourceSpan {
                start: self.current().span.start,
                end: self.current().span.start,
            })
    }

    fn span_from(&self, start: SourcePosition) -> SourceSpan {
        SourceSpan {
            start,
            end: self.previous_end(),
        }
    }

    fn error_current(&self, message: &str) -> String {
        let token = self.current();
        format!(
            "{} at {}:{} near {}",
            message,
            token.span.start.line,
            token.span.start.column,
            token.lexeme
        )
    }
}

fn render_module_ast(path: &Path, module: &ModuleAst) -> String {
    let mut lines = vec![
        "newcp-parser ast dump".to_string(),
        format!("input: {}", path.display()),
        format!("module: {}", module.name),
        format!("imports: {}", render_imports(&module.imports)),
        format!("declaration-count: {}", module.declarations.len()),
    ];

    for declaration in &module.declarations {
        render_declaration(declaration, 0, &mut lines);
    }
    if let Some(body) = &module.body {
        lines.push("BEGIN".to_string());
        render_statements(body, 1, &mut lines);
    }
    if let Some(close) = &module.close {
        lines.push("CLOSE".to_string());
        render_statements(close, 1, &mut lines);
    }

    lines.join("\n")
}

fn render_imports(imports: &[Import]) -> String {
    if imports.is_empty() {
        "<none>".to_string()
    } else {
        imports
            .iter()
            .map(|item| match &item.alias {
                Some(alias) => format!("{}:={}", alias, item.name),
                None => item.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn render_declaration(declaration: &Declaration, indent: usize, lines: &mut Vec<String>) {
    let prefix = "  ".repeat(indent);
    match declaration {
        Declaration::Const(item) => lines.push(format!(
            "{}const {} = {}",
            prefix,
            render_ident_def(&item.name),
            render_expr(&item.value)
        )),
        Declaration::Type(item) => lines.push(format!(
            "{}type {} = {}",
            prefix,
            render_ident_def(&item.name),
            render_type(&item.ty)
        )),
        Declaration::Var(item) => lines.push(format!(
            "{}var {}: {}",
            prefix,
            item.names.iter().map(render_ident_def).collect::<Vec<_>>().join(", "),
            render_type(&item.ty)
        )),
        Declaration::Forward(item) => lines.push(format!(
            "{}forward {}",
            prefix,
            render_procedure_heading(&item.heading)
        )),
        Declaration::Procedure(item) => {
            lines.push(format!(
                "{}procedure {}",
                prefix,
                render_procedure_heading(&item.heading)
            ));
            if let Some(body) = &item.body {
                for nested in &body.declarations {
                    render_declaration(nested, indent + 1, lines);
                }
                if let Some(statements) = &body.body {
                    render_statements(statements, indent + 1, lines);
                }
            }
        }
    }
}

fn render_statements(statements: &[Statement], indent: usize, lines: &mut Vec<String>) {
    let prefix = "  ".repeat(indent);
    for statement in statements {
        match statement {
            Statement::Empty { .. } => lines.push(format!("{}empty", prefix)),
            Statement::Assignment { target, value, .. } => lines.push(format!(
                "{}assign {} := {}",
                prefix,
                render_designator(target),
                render_expr(value)
            )),
            Statement::ProcedureCall { designator, .. } => {
                lines.push(format!("{}call {}", prefix, render_designator(designator)))
            }
            Statement::Exit { .. } => lines.push(format!("{}exit", prefix)),
            Statement::Return { expr, .. } => lines.push(format!(
                "{}return {}",
                prefix,
                expr.as_ref().map(render_expr).unwrap_or_else(|| "<none>".to_string())
            )),
            Statement::If {
                branches,
                else_branch,
                ..
            } => {
                lines.push(format!("{}if", prefix));
                for branch in branches {
                    lines.push(format!("{}  branch {}", prefix, render_expr(&branch.condition)));
                    render_statements(&branch.body, indent + 2, lines);
                }
                if let Some(else_branch) = else_branch {
                    lines.push(format!("{}  else", prefix));
                    render_statements(else_branch, indent + 2, lines);
                }
            }
            Statement::Case {
                expr,
                arms,
                else_branch,
                ..
            } => {
                lines.push(format!("{}case {}", prefix, render_expr(expr)));
                for arm in arms {
                    lines.push(format!(
                        "{}  arm {}",
                        prefix,
                        arm.labels.iter().map(render_case_label).collect::<Vec<_>>().join(", ")
                    ));
                    render_statements(&arm.body, indent + 2, lines);
                }
                if let Some(else_branch) = else_branch {
                    lines.push(format!("{}  else", prefix));
                    render_statements(else_branch, indent + 2, lines);
                }
            }
            Statement::While { condition, body, .. } => {
                lines.push(format!("{}while {}", prefix, render_expr(condition)));
                render_statements(body, indent + 1, lines);
            }
            Statement::Repeat { body, until, .. } => {
                lines.push(format!("{}repeat", prefix));
                render_statements(body, indent + 1, lines);
                lines.push(format!("{}until {}", prefix, render_expr(until)));
            }
            Statement::For {
                variable,
                start,
                end,
                step,
                body,
                ..
            } => {
                lines.push(format!(
                    "{}for {} := {} to {}{}",
                    prefix,
                    variable,
                    render_expr(start),
                    render_expr(end),
                    step.as_ref().map(|item| format!(" by {}", render_expr(item))).unwrap_or_default()
                ));
                render_statements(body, indent + 1, lines);
            }
            Statement::Loop { body, .. } => {
                lines.push(format!("{}loop", prefix));
                render_statements(body, indent + 1, lines);
            }
            Statement::With { arms, else_branch, .. } => {
                lines.push(format!("{}with", prefix));
                for arm in arms {
                    lines.push(format!(
                        "{}  arm {}",
                        prefix,
                        arm.guard.as_ref().map(render_guard).unwrap_or_else(|| "<empty>".to_string())
                    ));
                    render_statements(&arm.body, indent + 2, lines);
                }
                if let Some(else_branch) = else_branch {
                    lines.push(format!("{}  else", prefix));
                    render_statements(else_branch, indent + 2, lines);
                }
            }
        }
    }
}

fn render_case_label(label: &CaseLabel) -> String {
    match &label.end {
        Some(end) => format!("{}..{}", render_expr(&label.start), render_expr(end)),
        None => render_expr(&label.start),
    }
}

fn render_guard(guard: &Guard) -> String {
    format!("{}:{}", render_qualident(&guard.variable), render_qualident(&guard.ty))
}

fn render_procedure_heading(heading: &ProcedureHeading) -> String {
    let receiver = heading.receiver.as_ref().map(render_receiver).unwrap_or_default();
    let formal_parameters = heading.formal_parameters.as_ref().map(render_formal_parameters).unwrap_or_default();
    let sys_flag = heading
        .sys_flag
        .as_ref()
        .map(render_sys_flag)
        .map(|item| format!("{item} "))
        .unwrap_or_default();
    let attributes = render_method_attributes(&heading.attributes);
    format!(
        "{}{}{}{}",
        format!("{sys_flag}{receiver}"),
        render_ident_def(&heading.name),
        formal_parameters,
        attributes
    )
}

fn render_receiver(receiver: &Receiver) -> String {
    let mode = match receiver.mode {
        Some(ParamMode::Var) => "VAR ",
        Some(ParamMode::In) => "IN ",
        Some(ParamMode::Out) => "OUT ",
        None => "",
    };
    format!("({}{}:{}) ", mode, receiver.name, receiver.ty)
}

fn render_formal_parameters(parameters: &FormalParameters) -> String {
    let sections = parameters
        .sections
        .iter()
        .map(|section| {
            let mode = match section.mode {
                Some(ParamMode::Var) => "VAR ",
                Some(ParamMode::In) => "IN ",
                Some(ParamMode::Out) => "OUT ",
                None => "",
            };
            let sys_flag = section
                .sys_flag
                .as_ref()
                .map(render_sys_flag)
                .map(|item| format!("{item} "))
                .unwrap_or_default();
            format!(
                "{}{}: {}",
                format!("{mode}{sys_flag}"),
                section.names.join(", "),
                render_type(&section.ty)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    match &parameters.result_type {
        Some(result) => format!("({}): {}", sections, render_type(result)),
        None => format!("({})", sections),
    }
}

fn render_method_attributes(attributes: &MethodAttributes) -> String {
    let mut parts = Vec::new();
    if attributes.is_new {
        parts.push("NEW".to_string());
    }
    if let Some(flavor) = attributes.flavor {
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
}

fn render_sys_flag(flag: &SysFlag) -> String {
    match flag {
        SysFlag::Named(name) => format!("[{name}]"),
        SysFlag::Numeric(value) => format!("[{value}]"),
    }
}

fn render_ident_def(ident: &IdentDef) -> String {
    match ident.export {
        Some(ExportMark::Exported) => format!("{}*", ident.name),
        Some(ExportMark::ReadOnly) => format!("{}-", ident.name),
        None => ident.name.clone(),
    }
}

fn render_qualident(ident: &QualIdent) -> String {
    match &ident.module {
        Some(module) => format!("{}.{}", module, ident.name),
        None => ident.name.clone(),
    }
}

fn render_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::QualIdent { ident, .. } => render_qualident(ident),
        TypeExpr::Array {
            sys_flag,
            lengths,
            element_type,
            ..
        } => {
            let sys_flag = sys_flag
                .as_ref()
                .map(render_sys_flag)
                .map(|item| format!("{item} "))
                .unwrap_or_default();
            let prefix = if lengths.is_empty() {
                String::new()
            } else {
                format!("{} ", lengths.iter().map(render_expr).collect::<Vec<_>>().join(", "))
            };
            format!("ARRAY {}{}OF {}", sys_flag, prefix, render_type(element_type))
        }
        TypeExpr::Record {
            flavor,
            sys_flag,
            base,
            fields,
            ..
        } => {
            let flavor = flavor
                .map(|item| match item {
                    RecordFlavor::Abstract => "ABSTRACT ",
                    RecordFlavor::Extensible => "EXTENSIBLE ",
                    RecordFlavor::Limited => "LIMITED ",
                })
                .unwrap_or("");
            let sys_flag = sys_flag
                .as_ref()
                .map(render_sys_flag)
                .map(|item| format!("{item} "))
                .unwrap_or_default();
            let base = base
                .as_ref()
                .map(|item| format!("({}) ", render_qualident(item)))
                .unwrap_or_default();
            let fields = if fields.is_empty() {
                String::new()
            } else {
                fields
                    .iter()
                    .map(|field| {
                        format!(
                            "{}: {}",
                            field.names.iter().map(render_ident_def).collect::<Vec<_>>().join(", "),
                            render_type(&field.ty)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            format!("{}RECORD {}{}{}END", flavor, sys_flag, base, fields)
        }
        TypeExpr::Pointer { sys_flag, target, .. } => {
            let sys_flag = sys_flag
                .as_ref()
                .map(render_sys_flag)
                .map(|item| format!(" {item}"))
                .unwrap_or_default();
            format!("POINTER{} TO {}", sys_flag, render_type(target))
        }
        TypeExpr::Procedure {
            sys_flag,
            formal_parameters,
            ..
        } => match formal_parameters {
            Some(parameters) => {
                let sys_flag = sys_flag
                    .as_ref()
                    .map(render_sys_flag)
                    .map(|item| format!(" {item}"))
                    .unwrap_or_default();
                format!("PROCEDURE{} {}", sys_flag, render_formal_parameters(parameters))
            }
            None => sys_flag
                .as_ref()
                .map(render_sys_flag)
                .map(|item| format!("PROCEDURE {item}"))
                .unwrap_or_else(|| "PROCEDURE".to_string()),
        },
    }
}

fn render_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal { value, .. } => match value {
            Literal::Integer(value)
            | Literal::Real(value)
            | Literal::Character(value)
            | Literal::String(value) => value.clone(),
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
                UnaryOp::Plus => "+",
                UnaryOp::Minus => "-",
                UnaryOp::Not => "~",
            },
            render_expr(expr)
        ),
        Expr::Binary {
            left, op, right, ..
        } => format!(
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

fn expr_start(expr: &Expr) -> SourcePosition {
    expr_span(expr).start
}

fn expr_end(expr: &Expr) -> SourcePosition {
    expr_span(expr).end
}

fn expr_span(expr: &Expr) -> SourceSpan {
    match expr {
        Expr::Literal { span, .. }
        | Expr::Nil { span }
        | Expr::Set { span, .. }
        | Expr::Unary { span, .. }
        | Expr::Binary { span, .. } => *span,
        Expr::Designator(designator) => designator.span,
    }
}

fn statement_span(statement: &Statement) -> SourceSpan {
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
        | Statement::Return { span, .. } => *span,
    }
}

fn sequence_end(statements: &[Statement]) -> Option<SourcePosition> {
    statements.last().map(|statement| statement_span(statement).end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_with_arm_without_guard() {
        let module = parse_module_ast(
            "MODULE Demo;\nPROCEDURE Run;\nBEGIN\nWITH rec: T DO | ELSE END\nEND Run;\nEND Demo.",
        )
        .expect("module should parse");

        let body = module
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Procedure(procedure) => procedure.body.as_ref(),
                _ => None,
            })
            .and_then(|body| body.body.as_ref())
            .expect("procedure body");

        let Statement::With { arms, else_branch, .. } = &body[0] else {
            panic!("expected WITH statement");
        };
        assert_eq!(arms.len(), 2);
        assert!(arms[0].guard.is_some());
        assert!(arms[1].guard.is_none());
        assert!(arms[1].body.is_empty());
        assert!(else_branch.is_some());
    }

    #[test]
    fn parses_simple_module_source() {
        let source = "MODULE HostMenus;\nIMPORT Kernel;\nCONST Version* = 1;\nTYPE Menu* = RECORD END;\nVAR Current* : INTEGER;\nPROCEDURE OpenApp*;\nBEGIN\nEND OpenApp;\nPROCEDURE Install*(x: INTEGER);\nBEGIN\nEND Install;\nBEGIN\nEND HostMenus.";
        let spec = parse_source_module(source).expect("module should parse");

        assert_eq!(spec.name, "HostMenus");
        assert_eq!(spec.imports, vec!["Kernel"]);
        assert_eq!(
            spec.exports,
            vec![
                SourceExport {
                    name: "Version".to_string(),
                    kind: SourceExportKind::Constant,
                },
                SourceExport {
                    name: "Menu".to_string(),
                    kind: SourceExportKind::Type,
                },
                SourceExport {
                    name: "Current".to_string(),
                    kind: SourceExportKind::Variable,
                },
                SourceExport {
                    name: "OpenApp".to_string(),
                    kind: SourceExportKind::Command,
                },
                SourceExport {
                    name: "Install".to_string(),
                    kind: SourceExportKind::Procedure,
                },
            ]
        );
        assert_eq!(
            spec.procedures,
            vec![
                SourceProcedure {
                    name: "OpenApp".to_string(),
                    exported: true,
                    has_parameters: false,
                },
                SourceProcedure {
                    name: "Install".to_string(),
                    exported: true,
                    has_parameters: true,
                },
            ]
        );
        assert_eq!(spec.command_exports(), vec!["OpenApp"]);
    }

    #[test]
    fn parses_import_aliases_readonly_exports_and_method_headings() {
        let source = "MODULE Demo;\nIMPORT Fs := Files, Kernel;\nVAR Current-: INTEGER;\nPROCEDURE (node: Tree) Visit-;\nBEGIN\nEND Visit;\nBEGIN\nEND Demo.";
        let spec = parse_source_module(source).expect("module should parse");

        assert_eq!(spec.imports, vec!["Files", "Kernel"]);
        assert_eq!(
            spec.exports,
            vec![
                SourceExport {
                    name: "Current".to_string(),
                    kind: SourceExportKind::Variable,
                },
                SourceExport {
                    name: "Visit".to_string(),
                    kind: SourceExportKind::Command,
                },
            ]
        );
        assert_eq!(
            spec.procedures,
            vec![SourceProcedure {
                name: "Visit".to_string(),
                exported: true,
                has_parameters: false,
            }]
        );
    }

    #[test]
    fn parses_real_module_ast_with_nested_statements_and_types() {
        let source = "MODULE Demo;\nIMPORT Log := StdLog;\nTYPE Tree* = POINTER TO Node; Node = EXTENSIBLE RECORD (Base.Node) left, right: Tree; name-: ARRAY OF CHAR END;\nVAR root*: Tree;\nPROCEDURE Insert*(name: ARRAY OF CHAR), NEW, EXTENSIBLE;\nVAR i: INTEGER;\nBEGIN\n  IF root = NIL THEN root := NIL ELSE root.name := name$ END;\n  WHILE i < 10 DO INC(i) END;\n  RETURN\nEND Insert;\nBEGIN\n  Insert(\"x\")\nCLOSE\n  Log.String(\"bye\")\nEND Demo.";

        let module = parse_module_ast(source).expect("module should parse");
        assert_eq!(module.name, "Demo");
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.declarations.len(), 4);

        let Declaration::Procedure(procedure) = &module.declarations[3] else {
            panic!("expected procedure declaration");
        };
        assert!(procedure.heading.attributes.is_new);
        assert_eq!(procedure.heading.attributes.flavor, Some(MethodFlavor::Extensible));
        let body = procedure.body.as_ref().expect("procedure body");
        assert_eq!(body.declarations.len(), 1);
        assert_eq!(body.body.as_ref().expect("statements").len(), 3);
        assert_eq!(module.body.as_ref().expect("module body").len(), 1);
        assert_eq!(module.close.as_ref().expect("close body").len(), 1);
    }

    #[test]
    fn parses_header_only_abstract_and_empty_methods() {
        let source = "MODULE Demo;\nTYPE Base = ABSTRACT RECORD END;\nPROCEDURE (self: Base) Draw(), NEW, ABSTRACT;\nPROCEDURE (self: Base) Notify(), NEW, EMPTY;\nEND Demo.";

        let module = parse_module_ast(source).expect("module should parse");
        assert_eq!(module.declarations.len(), 3);

        let methods = module
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Procedure(procedure) => Some(procedure),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0].heading.name.name, "Draw");
        assert_eq!(methods[0].heading.attributes.flavor, Some(MethodFlavor::Abstract));
        assert!(methods[0].body.is_none());
        assert_eq!(methods[1].heading.name.name, "Notify");
        assert_eq!(methods[1].heading.attributes.flavor, Some(MethodFlavor::Empty));
        assert!(methods[1].body.is_none());
    }

    #[test]
    fn ast_dump_renders_tree_shape() {
        let temp = std::env::temp_dir().join("newcp-parser-ast.cp");
        std::fs::write(
            &temp,
            "MODULE Demo;\nVAR Current*: INTEGER;\nBEGIN\nCurrent := 1\nEND Demo.",
        )
        .expect("write test module");

        let dump = dump_ast(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("newcp-parser ast dump"));
        assert!(dump.contains("var Current*: INTEGER"));
        assert!(dump.contains("assign Current := 1"));
    }

    #[test]
    fn preserves_ambiguous_parenthesized_selector_for_sema() {
        let source = "MODULE Demo; BEGIN node(T) END Demo.";
        let module = parse_module_ast(source).expect("module should parse");
        let body = module.body.as_ref().expect("module body");

        let Statement::ProcedureCall { designator, .. } = &body[0] else {
            panic!("expected procedure call statement");
        };
        assert!(matches!(
            designator.selectors.as_slice(),
            [Selector::AmbiguousParen(qualident)] if qualident.name == "T"
        ));
    }

    #[test]
    fn parses_multi_arm_case_and_with_statements() {
        let source = concat!(
            "MODULE Demo;\n",
            "PROCEDURE Run(node: Base; p: POINTER TO Base; x: INTEGER);\n",
            "BEGIN\n",
            "  CASE x OF 0: Inc(x) | 1: Dec(x) ELSE x := 0 END;\n",
            "  WITH node: Child DO Inc(x) | p: Child DO Dec(x) ELSE x := 0 END\n",
            "END Run;\n",
            "END Demo."
        );

        let module = parse_module_ast(source).expect("module should parse");
        let Declaration::Procedure(procedure) = &module.declarations[0] else {
            panic!("expected procedure declaration");
        };
        let body = procedure
            .body
            .as_ref()
            .and_then(|body| body.body.as_ref())
            .expect("procedure statements");

        let Statement::Case { arms, else_branch, .. } = &body[0] else {
            panic!("expected case statement");
        };
        assert_eq!(arms.len(), 2);
        assert!(else_branch.is_some());
        let Statement::With { arms, else_branch, .. } = &body[1] else {
            panic!("expected with statement");
        };
        assert_eq!(arms.len(), 2);
        assert!(else_branch.is_some());
    }

    #[test]
    fn parses_system_flags_in_type_and_procedure_positions() {
        let source = concat!(
            "MODULE Demo;\n",
            "IMPORT SYSTEM;\n",
            "TYPE RawPtr = POINTER [untagged] TO RawRec;\n",
            "RawRec = RECORD [align8] value: INTEGER END;\n",
            "ByteBuf = ARRAY [noalign] 16 OF BYTE;\n",
            "Callback = PROCEDURE [ccall] (VAR [nil] p: RawPtr): INTEGER;\n",
            "PROCEDURE [ccall] Run(x: INTEGER);\n",
            "BEGIN\n",
            "END Run;\n",
            "END Demo."
        );

        let module = parse_module_ast(source).expect("module should parse");

        let Declaration::Type(raw_ptr) = &module.declarations[0] else {
            panic!("expected RawPtr type declaration");
        };
        let TypeExpr::Pointer { sys_flag, .. } = &raw_ptr.ty else {
            panic!("expected pointer type");
        };
        assert_eq!(sys_flag, &Some(SysFlag::Named("untagged".to_string())));

        let Declaration::Type(raw_rec) = &module.declarations[1] else {
            panic!("expected RawRec type declaration");
        };
        let TypeExpr::Record { sys_flag, .. } = &raw_rec.ty else {
            panic!("expected record type");
        };
        assert_eq!(sys_flag, &Some(SysFlag::Named("align8".to_string())));

        let Declaration::Type(byte_buf) = &module.declarations[2] else {
            panic!("expected ByteBuf type declaration");
        };
        let TypeExpr::Array { sys_flag, .. } = &byte_buf.ty else {
            panic!("expected array type");
        };
        assert_eq!(sys_flag, &Some(SysFlag::Named("noalign".to_string())));

        let Declaration::Type(callback) = &module.declarations[3] else {
            panic!("expected Callback type declaration");
        };
        let TypeExpr::Procedure { sys_flag, formal_parameters, .. } = &callback.ty else {
            panic!("expected procedure type");
        };
        assert_eq!(sys_flag, &Some(SysFlag::Named("ccall".to_string())));
        let section = &formal_parameters.as_ref().expect("procedure type params").sections[0];
        assert_eq!(section.sys_flag, Some(SysFlag::Named("nil".to_string())));

        let Declaration::Procedure(run) = &module.declarations[4] else {
            panic!("expected procedure declaration");
        };
        assert_eq!(run.heading.sys_flag, Some(SysFlag::Named("ccall".to_string())));
    }

    #[test]
    fn parses_numeric_system_flags() {
        let source = "MODULE Demo; IMPORT SYSTEM; TYPE T = RECORD [-8] END; END Demo.";
        let module = parse_module_ast(source).expect("module should parse");
        let Declaration::Type(type_decl) = &module.declarations[0] else {
            panic!("expected type declaration");
        };
        let TypeExpr::Record { sys_flag, .. } = &type_decl.ty else {
            panic!("expected record type");
        };
        assert_eq!(sys_flag, &Some(SysFlag::Numeric(-8)));
    }

    #[test]
    fn ast_nodes_capture_source_spans() {
        let source = "MODULE Demo;\nVAR Current*: INTEGER;\nBEGIN\nCurrent := 1\nEND Demo.";
        let module = parse_module_ast(source).expect("module should parse");

        assert_eq!(module.span.start.line, 1);
        assert_eq!(module.span.end.line, 5);

        let Declaration::Var(var_decl) = &module.declarations[0] else {
            panic!("expected var declaration");
        };
        assert_eq!(var_decl.span.start.line, 2);
        assert_eq!(var_decl.names[0].span.start.column, 5);

        let body = module.body.as_ref().expect("module body");
        let Statement::Assignment { span, target, value } = &body[0] else {
            panic!("expected assignment statement");
        };
        assert_eq!(span.start.line, 4);
        assert_eq!(target.span.start.column, 1);
        assert_eq!(expr_end(value).column, 13);
    }
}
