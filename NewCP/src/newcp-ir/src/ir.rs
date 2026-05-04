use crate::types::IrType;

/// Unique identifier for a basic block within a procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl BlockId {
    pub fn render(self) -> String {
        format!("bb{}", self.0)
    }
}

/// Unique identifier for a temporary value (SSA-style name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TempId(pub u32);

impl TempId {
    pub fn render(self) -> String {
        format!("t{}", self.0)
    }
}

/// A value that can appear as an instruction operand.
#[derive(Debug, Clone, PartialEq)]
pub enum IrValue {
    /// A temporary defined earlier in this procedure.
    Temp(TempId, IrType),
    /// An integer constant.
    ConstInt(i128, IrType),
    /// A floating-point constant.
    ConstReal(f64, IrType),
    /// A boolean constant.
    ConstBool(bool),
    /// A character constant (Unicode scalar).
    ConstChar(char),
    /// A string constant (interned literal).
    ConstStr(String),
    /// The null pointer.
    Null(IrType),
    /// Reference to a named symbol in this module (global variable, procedure).
    GlobalRef(String, IrType),
    /// Reference to a named symbol from an imported module: `Module.Name`.
    ImportRef(String, String, IrType),
}

impl IrValue {
    pub fn ty(&self) -> IrType {
        match self {
            IrValue::Temp(_, ty) => ty.clone(),
            IrValue::ConstInt(_, ty) => ty.clone(),
            IrValue::ConstReal(_, ty) => ty.clone(),
            IrValue::ConstBool(_) => IrType::Bool,
            IrValue::ConstChar(_) => IrType::Char,
            IrValue::ConstStr(_) => IrType::Ptr(Box::new(IrType::ShortChar)),
            IrValue::Null(ty) => ty.clone(),
            IrValue::GlobalRef(_, ty) => ty.clone(),
            IrValue::ImportRef(_, _, ty) => ty.clone(),
        }
    }

    pub fn render(&self) -> String {
        match self {
            IrValue::Temp(id, _) => id.render(),
            IrValue::ConstInt(v, _) => v.to_string(),
            IrValue::ConstReal(v, _) => format!("{v}"),
            IrValue::ConstBool(b) => b.to_string(),
            IrValue::ConstChar(c) => format!("'{c}'"),
            IrValue::ConstStr(s) => format!("\"{s}\""),
            IrValue::Null(_) => "null".to_string(),
            IrValue::GlobalRef(name, _) => name.clone(),
            IrValue::ImportRef(module, name, _) => format!("{module}.{name}"),
        }
    }
}

/// Binary operation kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    And, Or, Xor, Shl, Shr,
    Eq, Ne, Lt, Le, Gt, Ge,
    In,  // set membership
}

impl BinOp {
    pub fn render(self) -> &'static str {
        match self {
            BinOp::Add => "add", BinOp::Sub => "sub", BinOp::Mul => "mul",
            BinOp::Div => "div", BinOp::Mod => "mod",
            BinOp::And => "and", BinOp::Or => "or", BinOp::Xor => "xor",
            BinOp::Shl => "shl", BinOp::Shr => "shr",
            BinOp::Eq => "eq", BinOp::Ne => "ne", BinOp::Lt => "lt",
            BinOp::Le => "le", BinOp::Gt => "gt", BinOp::Ge => "ge",
            BinOp::In => "in",
        }
    }
}

/// Unary operation kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg, Not, BitNot,
}

impl UnOp {
    pub fn render(self) -> &'static str {
        match self {
            UnOp::Neg => "neg", UnOp::Not => "not", UnOp::BitNot => "bitnot",
        }
    }
}

/// Trap kind — the reason for an unconditional runtime abort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapKind {
    Assert,       // ASSERT(cond) — condition was false
    Halt(i32),    // HALT(code)
    NilDeref,     // implicit nil pointer dereference check
    ArrayBounds,  // array index out of bounds
    TypeGuard,    // WITH/IS guard failed (ELSE branch absent)
    CaseFallthrough, // CASE with no matching arm and no ELSE
}

impl TrapKind {
    pub fn render(&self) -> String {
        match self {
            TrapKind::Assert => "assert".to_string(),
            TrapKind::Halt(code) => format!("halt({code})"),
            TrapKind::NilDeref => "nil_deref".to_string(),
            TrapKind::ArrayBounds => "array_bounds".to_string(),
            TrapKind::TypeGuard => "type_guard".to_string(),
            TrapKind::CaseFallthrough => "case_fallthrough".to_string(),
        }
    }
}

/// A single instruction (non-terminating).
#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    /// `t = BinOp a, b`
    BinOp { dst: TempId, op: BinOp, left: IrValue, right: IrValue, ty: IrType },
    /// `t = UnOp a`
    UnOp { dst: TempId, op: UnOp, operand: IrValue, ty: IrType },
    /// `t = load addr`
    Load { dst: TempId, addr: IrValue, ty: IrType },
    /// `t = load_raw addr`
    LoadRaw { dst: TempId, addr: IrValue, ty: IrType },
    /// `store addr, value`
    Store { addr: IrValue, value: IrValue },
    /// `store_raw addr, value`
    StoreRaw { addr: IrValue, value: IrValue },
    /// `t = call f(args)`
    Call { dst: Option<TempId>, callee: IrValue, args: Vec<IrValue>, ret_ty: IrType },
    /// `t = methodcall descriptor, slot, args`
    MethodCall { dst: Option<TempId>, descriptor: IrValue, slot: u32, args: Vec<IrValue>, ret_ty: IrType },
    /// `t = addrof sym`  — SYSTEM.ADR, or taking address of a VAR slot
    AddrOf { dst: TempId, sym: IrValue },
    /// `t = bitcast value to ty`  — SYSTEM.VAL
    BitCast { dst: TempId, value: IrValue, ty: IrType },
    /// `t = lsh value, shift`  — SYSTEM.LSH
    Lsh { dst: TempId, value: IrValue, shift: IrValue, ty: IrType },
    /// `t = ash value, shift`  — ASH builtin (arithmetic shift: shl for n≥0, ashr for n<0)
    Ash { dst: TempId, value: IrValue, shift: IrValue, ty: IrType },
    /// `t = rot value, shift`  — SYSTEM.ROT
    Rot { dst: TempId, value: IrValue, shift: IrValue, ty: IrType },
    /// `memcopy dst, src, len`  — SYSTEM.MOVE
    MemCopy { dst: IrValue, src: IrValue, len: IrValue },
    /// `t = typ value`  — SYSTEM.TYP
    TypTag { dst: TempId, value: IrValue },
    /// `t = sysnew size`  — SYSTEM.NEW
    SysNew { dst: TempId, size: IrValue },
    /// `t = typecheck value is ty`  — IS expression (boolean result)
    TypeCheck { dst: TempId, value: IrValue, ty: IrType },
    /// `t = gep base, field_index`  — struct field pointer (GEP into a record)
    ///
    /// `base` is a pointer to the record. `field_index` is the zero-based
    /// field index in the flattened field list (accounting for inherited fields).
    /// `result_ty` is the type of the field (value type, not pointer).
    Gep { dst: TempId, base: IrValue, field_index: u32, result_ty: IrType },
    /// `t = new RecordType`  — allocate a new GC-managed record of the given type.
    ///
    /// `record_ty` is the IR type of the record (usually `Named("T")`). The backend
    /// computes `sizeof(T)` and calls `__newcp_sys_new`; the result is a `ptr`.
    New { dst: TempId, record_ty: IrType },
    /// `store result_slot, value`  — internal: prepare RETURN value before Br function_exit
    StoreResult { value: IrValue },
}

/// The terminating instruction of a basic block.
/// Every block has exactly one terminator.
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    /// Unconditional branch.
    Br { target: BlockId },
    /// Conditional branch.
    CondBr { cond: IrValue, true_target: BlockId, false_target: BlockId },
    /// Logical RETURN (non-void). Lowered to StoreResult + Br(function_exit).
    Ret { value: IrValue },
    /// Logical RETURN (void). Lowered to Br(function_exit).
    RetVoid,
    /// Runtime trap — unconditional abort with a reason code.
    Trap { kind: TrapKind },
    /// WITH/IS type test branch.
    /// Branches to `true_target` if `value`'s dynamic type extends `ty`,
    /// otherwise `false_target`.
    TypeTest { value: IrValue, ty: IrType, true_target: BlockId, false_target: BlockId },
}

/// A single basic block.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    /// Order in which this block was constructed (0-based).
    pub construction_index: u32,
    /// Reverse post-order index, set after the CFG is complete.
    pub rpo_index: Option<u32>,
    pub instrs: Vec<Instr>,
    pub terminator: Terminator,
}

impl BasicBlock {
    pub fn render_header(&self) -> String {
        let rpo = self
            .rpo_index
            .map(|r| format!(" rpo={r}"))
            .unwrap_or_default();
        format!("{}  [c={}{rpo}]:", self.id.render(), self.construction_index)
    }
}
