use inkwell::context::Context;
use inkwell::types::{AnyTypeEnum, BasicTypeEnum};

use newcp_ir::IrType;

use crate::error::CodegenError;

/// Lowers `IrType` values to LLVM types.
///
/// Constructed once per `CodegenModule` and borrowed during emission.
pub struct TypeLowerer<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeLowerer<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Lower an `IrType` to a LLVM `BasicTypeEnum`.
    ///
    /// Returns `Unsupported` for types that require layout knowledge not yet
    /// available in the first slice (e.g. `Named`, `Opaque` record shapes).
    pub fn lower_basic(&self, ty: &IrType) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        match ty {
            IrType::I8 | IrType::U8 => Ok(self.context.i8_type().into()),
            IrType::I16 | IrType::U16 => Ok(self.context.i16_type().into()),
            IrType::I32 | IrType::U32 => Ok(self.context.i32_type().into()),
            IrType::I64 => Ok(self.context.i64_type().into()),
            IrType::F32 => Ok(self.context.f32_type().into()),
            IrType::F64 => Ok(self.context.f64_type().into()),
            // Bool lowers to i1.
            IrType::Bool => Ok(self.context.bool_type().into()),
            // CHAR is 16-bit; SHORTCHAR is 8-bit.
            IrType::Char => Ok(self.context.i16_type().into()),
            IrType::ShortChar => Ok(self.context.i8_type().into()),
            // All pointer-bearing types lower to opaque `ptr`.
            IrType::Ptr(_) | IrType::UntaggedPtr(_) | IrType::Ref(_) => {
                Ok(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            // SET(32) → i32; other widths deferred.
            IrType::Set(32) => Ok(self.context.i32_type().into()),
            IrType::Set(w) => Err(CodegenError::Unsupported {
                stage: "type_lowering",
                detail: format!("SET({w}) is not yet supported; only SET(32)"),
            }),
            // Untagged records: deferred until layout pass is implemented.
            IrType::UntaggedRecord { name, .. } => Err(CodegenError::Unsupported {
                stage: "type_lowering",
                detail: format!("UntaggedRecord '{name}' layout lowering not yet implemented"),
            }),
            // Named and Opaque: conservative — lower to ptr in first slice only
            // when the caller explicitly requests a pointer-shaped fallback.
            // Direct use as a value type is unsupported until ABI is known.
            IrType::Named(name) => Err(CodegenError::Unsupported {
                stage: "type_lowering",
                detail: format!("Named type '{name}' requires layout knowledge not yet available"),
            }),
            IrType::Opaque(name) => {
                // Opaque types appear as runtime descriptors passed as pointers.
                // Lower to ptr as a safe placeholder.
                let _ = name;
                Ok(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            IrType::Void => Err(CodegenError::Unsupported {
                stage: "type_lowering",
                detail: "Void is not a basic type; use lower_return_type instead".to_string(),
            }),
        }
    }

    /// Lower an `IrType` for use as a function return type.
    pub fn lower_return_type(&self, ty: &IrType) -> Result<AnyTypeEnum<'ctx>, CodegenError> {
        if *ty == IrType::Void {
            return Ok(self.context.void_type().into());
        }
        let basic = self.lower_basic(ty)?;
        Ok(match basic {
            BasicTypeEnum::ArrayType(t) => t.into(),
            BasicTypeEnum::FloatType(t) => t.into(),
            BasicTypeEnum::IntType(t) => t.into(),
            BasicTypeEnum::PointerType(t) => t.into(),
            BasicTypeEnum::StructType(t) => t.into(),
            BasicTypeEnum::VectorType(t) => t.into(),
            BasicTypeEnum::ScalableVectorType(t) => t.into(),
        })
    }
}
