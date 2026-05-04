use std::collections::HashMap;

use inkwell::context::Context;
use inkwell::types::{AnyTypeEnum, BasicTypeEnum, BasicType, StructType};

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
    /// `named_types`: optional map from type name to a declared LLVM `StructType`.
    /// When provided, `Named` types are resolved through this map.
    pub fn lower_basic(
        &self,
        ty: &IrType,
        named_types: Option<&HashMap<String, StructType<'ctx>>>,
    ) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
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
            // Fixed-length array: lower to LLVM [N x T].
            IrType::Array { element, len } => {
                let elem_ty = self.lower_basic(element, named_types)?;
                Ok(elem_ty.array_type(*len as u32).into())
            }
            // SET(32) → i32; other widths deferred.
            IrType::Set(32) => Ok(self.context.i32_type().into()),
            IrType::Set(w) => Err(CodegenError::Unsupported {
                stage: "type_lowering",
                detail: format!("SET({w}) is not yet supported; only SET(32)"),
            }),
            // Untagged records: lower to opaque `ptr` in the first slice.
            IrType::UntaggedRecord { .. } => {
                Ok(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            // Named: look up in declared struct types if provided.
            // If not found (e.g. pointer alias or forward ref), fall back to opaque `ptr`.
            IrType::Named(name) => {
                if let Some(map) = named_types {
                    if let Some(&st) = map.get(name.as_str()) {
                        return Ok(st.into());
                    }
                }
                // Not a known struct — pointer alias or opaque forward reference.
                Ok(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            IrType::Opaque(_) => {
                Ok(self.context.ptr_type(inkwell::AddressSpace::default()).into())
            }
            IrType::Void => Err(CodegenError::Unsupported {
                stage: "type_lowering",
                detail: "Void is not a basic type; use lower_return_type instead".to_string(),
            }),
        }
    }

    /// Lower an `IrType` for use as a function return type.
    pub fn lower_return_type(
        &self,
        ty: &IrType,
        named_types: Option<&HashMap<String, StructType<'ctx>>>,
    ) -> Result<AnyTypeEnum<'ctx>, CodegenError> {
        if *ty == IrType::Void {
            return Ok(self.context.void_type().into());
        }
        let basic = self.lower_basic(ty, named_types)?;
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
