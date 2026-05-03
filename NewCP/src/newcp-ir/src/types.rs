/// The IR type system.
///
/// Designed to carry enough information to map to LLVM IR without re-inferring
/// types, while keeping CP-specific distinctions (VAR params, sets, opaque
/// runtime types) explicit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrType {
    // Integer types — signed
    I8,
    I16,
    I32,
    I64,
    // Integer types — unsigned (BYTE, SHORTCHAR unsigned forms)
    U8,
    U16,
    U32,
    // Floating-point
    F32, // SHORTREAL
    F64, // REAL
    // Logical / character
    Bool,
    Char,      // CHAR (Latin-1, 16-bit in CP)
    ShortChar, // SHORTCHAR (8-bit)
    // Compound
    /// Pointer to T — a POINTER TO RECORD or any heap reference.
    Ptr(Box<IrType>),
    /// Reference to T — a VAR parameter; address-of semantics, not heap.
    Ref(Box<IrType>),
    /// Source-level named type: has an identity in the CP module graph.
    Named(String),
    /// Runtime-internal opaque type: descriptor headers, vtable arrays, tag
    /// words. Never exposed to language-level type checking.
    Opaque(String),
    /// CP SET type. `width` is the bit-width (32 for standard SET).
    Set(u8),
    Void,
}

impl IrType {
    pub fn render(&self) -> String {
        match self {
            IrType::I8 => "i8".to_string(),
            IrType::I16 => "i16".to_string(),
            IrType::I32 => "i32".to_string(),
            IrType::I64 => "i64".to_string(),
            IrType::U8 => "u8".to_string(),
            IrType::U16 => "u16".to_string(),
            IrType::U32 => "u32".to_string(),
            IrType::F32 => "f32".to_string(),
            IrType::F64 => "f64".to_string(),
            IrType::Bool => "bool".to_string(),
            IrType::Char => "char".to_string(),
            IrType::ShortChar => "shortchar".to_string(),
            IrType::Ptr(inner) => format!("ptr<{}>", inner.render()),
            IrType::Ref(inner) => format!("ref<{}>", inner.render()),
            IrType::Named(name) => format!("named:{name}"),
            IrType::Opaque(name) => format!("opaque:{name}"),
            IrType::Set(w) => format!("set{w}"),
            IrType::Void => "void".to_string(),
        }
    }
}
