use std::collections::HashMap;

use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::module::Linkage;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

use newcp_ir::{BinOp, BlockId, IrProcedure, IrType, IrValue, TempId, UnOp, OPEN_ARRAY_LEN_SUFFIX};

use crate::error::CodegenError;
use crate::module::CodegenModule;
use crate::options::CodegenOptions;

/// Per-procedure name resolution context.
///
/// Holds SSA temp values and the LLVM block map for one procedure.
/// Cleared between procedures — `TempId`s must not leak.
pub struct ValueMap<'ctx> {
    pub temp_values: HashMap<TempId, BasicValueEnum<'ctx>>,
    pub block_map: HashMap<BlockId, BasicBlock<'ctx>>,
    pub named_slots: HashMap<String, PointerValue<'ctx>>,
    pub ref_param_slots: HashMap<String, PointerValue<'ctx>>,
    /// The procedure's entry block — used for `alloca` placement.
    pub entry_block: Option<BasicBlock<'ctx>>,
}

impl<'ctx> ValueMap<'ctx> {
    fn new() -> Self {
        Self {
            temp_values: HashMap::new(),
            block_map: HashMap::new(),
            named_slots: HashMap::new(),
            ref_param_slots: HashMap::new(),
            entry_block: None,
        }
    }
}

/// Emits one procedure body into the `CodegenModule`.
pub struct ProcedureEmitter<'ctx, 'm> {
    cg: &'m mut CodegenModule<'ctx>,
}

impl<'ctx, 'm> ProcedureEmitter<'ctx, 'm> {
    pub fn new(cg: &'m mut CodegenModule<'ctx>, _options: &'m CodegenOptions) -> Self {
        Self { cg }
    }

    /// Stage 4: emit the body of `proc`.
    ///
    /// The declaration must already exist in `cg.planner.functions`.
    pub fn emit(&mut self, proc: &IrProcedure) -> Result<(), CodegenError> {
        let fn_val = self
            .cg
            .planner
            .functions
            .get(&proc.name)
            .copied()
            .ok_or_else(|| CodegenError::Unsupported {
                stage: "emit",
                detail: format!("procedure '{}' was not declared in planning stage", proc.name),
            })?;

        let mut value_map = ValueMap::new();

        // Pass 1: create all LLVM basic blocks.
        self.create_blocks(fn_val, proc, &mut value_map);
        self.bind_proc_slots(fn_val, proc, &mut value_map)?;

        // Pass 2: emit instructions and terminators.
        self.emit_blocks(fn_val, proc, &mut value_map)?;

        Ok(())
    }

    /// Pass 1: create one LLVM block per IR block, registering them in `value_map`.
    fn create_blocks(
        &self,
        fn_val: FunctionValue<'ctx>,
        proc: &IrProcedure,
        value_map: &mut ValueMap<'ctx>,
    ) {
        for block in &proc.blocks {
            let name = block.id.render();
            let llvm_block = self.cg.context.append_basic_block(fn_val, &name);
            value_map.block_map.insert(block.id, llvm_block);
        }
        value_map.entry_block = value_map.block_map.get(&proc.entry).copied();
    }

    /// Pass 2: position the builder at each block and emit its instructions.
    fn emit_blocks(
        &mut self,
        _fn_val: FunctionValue<'ctx>,
        proc: &IrProcedure,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        for block in &proc.blocks {
            let llvm_block = value_map.block_map[&block.id];
            self.cg.builder.position_at_end(llvm_block);

            for instr in &block.instrs {
                self.emit_instr(instr, value_map)?;
            }

            self.emit_terminator(&block.terminator, value_map)?;
        }
        Ok(())
    }

    fn emit_instr(
        &mut self,
        instr: &newcp_ir::Instr,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        use newcp_ir::Instr;

        match instr {
            Instr::Load { dst, addr, ty } => {
                let ptr = self.resolve_pointer(addr, value_map)?;
                let llvm_ty = self.lower_basic_type(ty)?;
                let value = self
                    .cg
                    .builder
                    .build_load(llvm_ty, ptr, &dst.render())
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                value_map.temp_values.insert(*dst, value);
                Ok(())
            }
            Instr::LoadRaw { dst, addr, ty } => {
                let ptr = self.resolve_raw_pointer(addr, ty, value_map)?;
                let llvm_ty = self.lower_basic_type(ty)?;
                let value = self
                    .cg
                    .builder
                    .build_load(llvm_ty, ptr, &dst.render())
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                value_map.temp_values.insert(*dst, value);
                Ok(())
            }
            Instr::Store { addr, value } => {
                let ptr = self.resolve_pointer(addr, value_map)?;
                let stored = self.resolve_basic_value(value, value_map)?;
                self.cg
                    .builder
                    .build_store(ptr, stored)
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                Ok(())
            }
            Instr::StoreRaw { addr, value } => {
                let ptr = self.resolve_raw_pointer(addr, &value.ty(), value_map)?;
                let stored = self.resolve_basic_value(value, value_map)?;
                self.cg
                    .builder
                    .build_store(ptr, stored)
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                Ok(())
            }
            Instr::StoreResult { value } => {
                let ptr = self.ensure_named_slot("$result", &value.ty(), value_map)?;
                let stored = self.resolve_basic_value(value, value_map)?;
                self.cg
                    .builder
                    .build_store(ptr, stored)
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                Ok(())
            }
            Instr::StringCompare {
                dst,
                lhs,
                rhs,
                op,
                elem_is_short,
            } => self.emit_string_compare_instr(*dst, lhs, rhs, *op, *elem_is_short, value_map),
            Instr::Safepoint => {
                let helper = self.get_or_declare_safepoint();
                self.cg
                    .builder
                    .build_call(helper, &[], "safepoint")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_safepoint",
                        detail: e.to_string(),
                    })?;
                Ok(())
            }
            Instr::UnOp { dst, op, operand, ty } => {
                let value = self.emit_unop(*op, operand, ty, value_map)?;
                value_map.temp_values.insert(*dst, value);
                Ok(())
            }
            Instr::BinOp {
                dst,
                op,
                left,
                right,
                ty,
            } => {
                let value = self.emit_binop(*op, left, right, ty, value_map)?;
                value_map.temp_values.insert(*dst, value);
                Ok(())
            }
            Instr::Call {
                dst,
                callee,
                args,
                ret_ty,
            } => {
                self.emit_call(*dst, callee, args, ret_ty, value_map)
            }
            Instr::AddrOf { dst, sym } => {
                // ConstStr: materialize the private string global, then take
                // its address. Used by lower_statement to memcpy a string
                // literal into a fixed-size CHAR/SHORTCHAR array.
                let ptr = match sym {
                    IrValue::ConstStr(s, elem_ty) => {
                        self.cg.get_or_emit_string_constant(s, elem_ty)
                    }
                    _ => self.resolve_pointer(sym, value_map)?,
                };
                let value = self
                    .cg
                    .builder
                    .build_ptr_to_int(ptr, self.cg.context.i64_type(), &dst.render())
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                value_map.temp_values.insert(*dst, value.into());
                Ok(())
            }
            Instr::BitCast { dst, value, ty } => {
                let cast = self.emit_bitcast(value, ty, value_map)?;
                value_map.temp_values.insert(*dst, cast);
                Ok(())
            }
            Instr::Cast { dst, value, to_ty } => {
                let result = self.emit_cast(value, to_ty, value_map)?;
                value_map.temp_values.insert(*dst, result);
                Ok(())
            }
            Instr::Lsh {
                dst,
                value,
                shift,
                ty,
            } => {
                let result = self.emit_lsh(value, shift, ty, value_map)?;
                value_map.temp_values.insert(*dst, result);
                Ok(())
            }
            Instr::Ash {
                dst,
                value,
                shift,
                ty,
            } => {
                let result = self.emit_ash(value, shift, ty, value_map)?;
                value_map.temp_values.insert(*dst, result);
                Ok(())
            }
            Instr::Rot {
                dst,
                value,
                shift,
                ty,
            } => {
                let result = self.emit_rot(value, shift, ty, value_map)?;
                value_map.temp_values.insert(*dst, result);
                Ok(())
            }
            Instr::Entier { dst, value } => {
                let fv = self.resolve_basic_value(value, value_map)?.into_float_value();
                let float_ty = fv.get_type();
                let bits = float_ty.get_bit_width();
                let intrinsic_name = if bits == 32 { "llvm.floor.f32" } else { "llvm.floor.f64" };
                let floor_fn_ty = float_ty.fn_type(&[float_ty.into()], false);
                let floor_fn = self.cg.module.get_function(intrinsic_name).unwrap_or_else(|| {
                    self.cg.module.add_function(intrinsic_name, floor_fn_ty, None)
                });
                let floored = self.cg.builder.build_call(floor_fn, &[fv.into()], "entier.floor")
                    .map_err(|e| CodegenError::Unsupported { stage: "emit_instr", detail: e.to_string() })?
                    .try_as_basic_value().unwrap_basic().into_float_value();
                let i64_ty = self.cg.context.i64_type();
                let result = self.cg.builder.build_float_to_signed_int(floored, i64_ty, "entier.cast")
                    .map_err(|e| CodegenError::Unsupported { stage: "emit_instr", detail: e.to_string() })?;
                value_map.temp_values.insert(*dst, result.into());
                Ok(())
            }
            Instr::MemCopy { dst, src, len } => {
                self.emit_memcopy(dst, src, len, value_map)
            }
            Instr::TypTag { .. } => Err(CodegenError::Unsupported {
                stage: "emit_instr",
                detail: "TypTag requires tagged-record TypeDesc lowering and heap/header ABI support".to_string(),
            }),
            Instr::SysNew { dst, size } => {
                let size_value = self.resolve_basic_value(size, value_map)?.into_int_value();
                let sys_new = self
                    .cg
                    .module
                    .get_function("__newcp_sys_new")
                    .ok_or_else(|| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: "__newcp_sys_new was not declared during planning".to_string(),
                    })?;
                let call = self
                    .cg
                    .builder
                    .build_call(sys_new, &[size_value.into()], &dst.render())
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })?;
                let value = call.try_as_basic_value().unwrap_basic();
                value_map.temp_values.insert(*dst, value);
                Ok(())
            }
            Instr::TypeCheck { dst, value, ty } => {
                let result = self.emit_type_check(value, ty, value_map)?;
                value_map.temp_values.insert(*dst, result);
                Ok(())
            }
            Instr::Gep { dst, base, field_index, result_ty } => {
                self.emit_gep(*dst, base, *field_index, result_ty, value_map)
            }
            Instr::IndexGep { dst, base, index, element_ty } => {
                self.emit_index_gep(*dst, base, index, element_ty, value_map)
            }
            Instr::New { dst, record_ty } => {
                // Three paths:
                //
                // - tagged record, type defined locally: emit a direct
                //   `__newcp_new_rec(@type.desc)` call against the
                //   compiled-in TypeDesc global.
                //
                // - tagged record, type defined in another CP module
                //   (qualified name e.g. "TextModels.StdModelDesc"):
                //   look up the TypeDesc address at runtime via the
                //   `__newcp_lookup_typedesc` registry helper. This
                //   avoids the cross-module linker symbol gymnastics
                //   that would be required to share TypeDesc globals
                //   between compiled CP modules.
                //
                // - untagged record (anonymous, no Named type): just
                //   allocate raw bytes via `__newcp_sys_new(size)`.
                let type_name = match record_ty {
                    newcp_ir::IrType::Named(n) => Some(n.as_str()),
                    _ => None,
                };
                let local_desc_global = type_name
                    .and_then(|n| self.cg.planner.type_desc_globals.get(n).copied());

                if let Some(desc_global) = local_desc_global {
                    let new_rec = self
                        .cg
                        .module
                        .get_function("__newcp_new_rec")
                        .ok_or_else(|| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: "__newcp_new_rec was not declared during planning".to_string(),
                        })?;
                    let desc_ptr = desc_global.as_pointer_value();
                    let call = self
                        .cg
                        .builder
                        .build_call(new_rec, &[desc_ptr.into()], &dst.render())
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    let value = call.try_as_basic_value().unwrap_basic();
                    value_map.temp_values.insert(*dst, value);
                } else if let Some(qualified) = type_name.filter(|n| n.contains('.')) {
                    // Cross-module typed allocation: look up the
                    // TypeDesc at runtime by qualified name, then
                    // dispatch through __newcp_new_rec.
                    let lookup = self.get_or_declare_lookup_typedesc();
                    let new_rec = self
                        .cg
                        .module
                        .get_function("__newcp_new_rec")
                        .ok_or_else(|| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: "__newcp_new_rec was not declared during planning".to_string(),
                        })?;
                    let name_global = self.cg.get_or_emit_cstring_constant(qualified);
                    let lookup_call = self
                        .cg
                        .builder
                        .build_call(lookup, &[name_global.into()], "typedesc")
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    let typedesc_i64 = lookup_call
                        .try_as_basic_value()
                        .unwrap_basic()
                        .into_int_value();
                    let typedesc_ptr = self
                        .cg
                        .builder
                        .build_int_to_ptr(
                            typedesc_i64,
                            self.cg.context.ptr_type(inkwell::AddressSpace::default()),
                            "typedesc.ptr",
                        )
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    let new_call = self
                        .cg
                        .builder
                        .build_call(new_rec, &[typedesc_ptr.into()], &dst.render())
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    let value = new_call.try_as_basic_value().unwrap_basic();
                    value_map.temp_values.insert(*dst, value);
                } else {
                    // No TypeDesc — fall back to raw allocation.
                    let struct_ty = named_struct_type_from_ir_type(record_ty, &self.cg.planner.named_struct_types)
                        .ok_or_else(|| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: format!("Instr::New: unknown record type {}", record_ty.render()),
                        })?;
                    let size_val = struct_ty.size_of().ok_or_else(|| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: format!("Instr::New: struct '{}' has no computable size", record_ty.render()),
                    })?;
                    let sys_new = self
                        .cg
                        .module
                        .get_function("__newcp_sys_new")
                        .ok_or_else(|| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: "__newcp_sys_new was not declared during planning".to_string(),
                        })?;
                    let call = self
                        .cg
                        .builder
                        .build_call(sys_new, &[size_val.into()], &dst.render())
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    let value = call.try_as_basic_value().unwrap_basic();
                    value_map.temp_values.insert(*dst, value);
                }
                Ok(())
            }
            Instr::MethodCall { dst, descriptor, slot, args, ret_ty } => {
                self.emit_method_call(*dst, descriptor, *slot, args, ret_ty, value_map)
            }
            Instr::NewArray { dst, elem_ty, len } => {
                self.emit_new_array(*dst, elem_ty, len, value_map)
            }
        }
    }

    /// `NEW(p, len)` for `POINTER TO ARRAY OF T`.
    ///
    /// Layout:
    /// ```text
    ///   alloc_base + 0..8   : header (i64 length)
    ///   alloc_base + 8..    : data (len * sizeof(elem_ty) bytes)
    /// ```
    /// Returns `alloc_base + 8` so callers index directly through the
    /// element pointer; `LEN(p^)` reads `*(p - 8)`.
    fn emit_new_array(
        &mut self,
        dst: TempId,
        elem_ty: &IrType,
        len: &IrValue,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        let i64_ty = self.cg.context.i64_type();
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());

        // Resolve element-type size in bytes via LLVM's size_of.
        let elem_llvm_ty = self.lower_basic_type(elem_ty)?;
        let elem_size: inkwell::values::IntValue<'ctx> = match elem_llvm_ty {
            inkwell::types::BasicTypeEnum::IntType(t) => t.size_of(),
            inkwell::types::BasicTypeEnum::FloatType(t) => t.size_of(),
            inkwell::types::BasicTypeEnum::PointerType(t) => t.size_of(),
            inkwell::types::BasicTypeEnum::StructType(t) => t.size_of().ok_or_else(|| {
                CodegenError::Unsupported {
                    stage: "emit_new_array",
                    detail: format!("struct element {elem_ty:?} has no size"),
                }
            })?,
            inkwell::types::BasicTypeEnum::ArrayType(t) => t.size_of().ok_or_else(|| {
                CodegenError::Unsupported {
                    stage: "emit_new_array",
                    detail: format!("array element {elem_ty:?} has no size"),
                }
            })?,
            other => {
                return Err(CodegenError::Unsupported {
                    stage: "emit_new_array",
                    detail: format!("element type {other:?} has no computable size"),
                });
            }
        };

        // total_bytes = len * elem_size + 8 (header)
        let len_val = self.resolve_basic_value(len, value_map)?.into_int_value();
        let len_i64 = if len_val.get_type().get_bit_width() == 64 {
            len_val
        } else {
            self.cg
                .builder
                .build_int_z_extend(len_val, i64_ty, "len_zx")
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_new_array",
                    detail: e.to_string(),
                })?
        };
        let bytes_for_data = self
            .cg
            .builder
            .build_int_mul(len_i64, elem_size, "data_bytes")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_new_array",
                detail: e.to_string(),
            })?;
        let header_bytes = i64_ty.const_int(8, false);
        let total_bytes = self
            .cg
            .builder
            .build_int_add(bytes_for_data, header_bytes, "total_bytes")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_new_array",
                detail: e.to_string(),
            })?;

        // Call __newcp_sys_new(total_bytes) -> ptr
        let sys_new = self
            .cg
            .module
            .get_function("__newcp_sys_new")
            .ok_or_else(|| CodegenError::Unsupported {
                stage: "emit_new_array",
                detail: "__newcp_sys_new was not declared during planning".to_string(),
            })?;
        let alloc = self
            .cg
            .builder
            .build_call(sys_new, &[total_bytes.into()], "dyn_arr_alloc")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_new_array",
                detail: e.to_string(),
            })?;
        let alloc_ptr = alloc
            .try_as_basic_value()
            .unwrap_basic()
            .into_pointer_value();

        // Store len at offset 0 (the header).
        self.cg
            .builder
            .build_store(alloc_ptr, len_i64)
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_new_array",
                detail: e.to_string(),
            })?;

        // Return alloc_ptr + 8 (skip the header) as the user-visible pointer.
        let i8_ty = self.cg.context.i8_type();
        let header_offset = i64_ty.const_int(8, false);
        let data_ptr = unsafe {
            self.cg
                .builder
                .build_gep(i8_ty, alloc_ptr, &[header_offset], &dst.render())
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_new_array",
                    detail: e.to_string(),
                })?
        };
        // The IR-level type for the result is `ptr` (just an opaque pointer
        // at LLVM level; the caller's IR type tracks the element type).
        value_map
            .temp_values
            .insert(dst, data_ptr.into());
        let _ = ptr_ty; // silence unused
        Ok(())
    }

    /// Emit a dynamic dispatch call through the object's TypeDesc vtable.
    ///
    /// Sequence:
    /// 1. GEP obj_ptr - 16 bytes → BlockHeader
    /// 2. Load `tag` (first field of BlockHeader, i64)
    /// 3. Mask low bit: `desc_ptr = tag & !1`
    /// 4. GEP into TypeDesc at field 4 (vtable) → load `vtable_ptr`
    /// 5. GEP vtable_ptr[slot] → load `fn_ptr`
    /// 6. Build indirect call with `receiver ++ args`
    fn emit_method_call(
        &mut self,
        dst: Option<newcp_ir::TempId>,
        receiver: &newcp_ir::IrValue,        slot: u32,
        args: &[newcp_ir::IrValue],
        ret_ty: &newcp_ir::IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        use inkwell::AddressSpace;
        use inkwell::types::BasicMetadataTypeEnum;
        use inkwell::values::BasicMetadataValueEnum;

        let i8_ty  = self.cg.context.i8_type();
        let i64_ty = self.cg.context.i64_type();
        let ptr_ty = self.cg.context.ptr_type(AddressSpace::default());

        // 1. Resolve the receiver pointer.
        let obj_ptr = self
            .resolve_basic_value(receiver, value_map)?
            .into_pointer_value();

        // 2. GEP obj_ptr[-16] → start of BlockHeader (tag is its first field).
        let neg16 = i64_ty.const_int((-16i64) as u64, false);
        let hdr_ptr = unsafe {
            self.cg
                .builder
                .build_gep(i8_ty, obj_ptr, &[neg16], "hdr_ptr")
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_method_call",
                    detail: e.to_string(),
                })?
        };

        // 3. Load tag (i64) and strip low bit to get TypeDesc*.
        let tag_val = self
            .cg
            .builder
            .build_load(i64_ty, hdr_ptr, "tag_raw")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_method_call",
                detail: e.to_string(),
            })?
            .into_int_value();
        let mask = i64_ty.const_int(!1u64, false);
        let tag_clean = self
            .cg
            .builder
            .build_and(tag_val, mask, "tag")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_method_call",
                detail: e.to_string(),
            })?;
        let desc_ptr = self
            .cg
            .builder
            .build_int_to_ptr(tag_clean, ptr_ty, "desc_ptr")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_method_call",
                detail: e.to_string(),
            })?;

        // 4. TypeDesc field 4 (vtable) is at byte offset 32 in the struct.
        //    Use a byte GEP rather than struct GEP to avoid needing the named struct.
        let offset32 = i64_ty.const_int(32, false);
        let vtable_field_ptr = unsafe {
            self.cg
                .builder
                .build_gep(i8_ty, desc_ptr, &[offset32], "vtable_field_ptr")
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_method_call",
                    detail: e.to_string(),
                })?
        };
        let vtable_ptr = self
            .cg
            .builder
            .build_load(ptr_ty, vtable_field_ptr, "vtable_ptr")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_method_call",
                detail: e.to_string(),
            })?
            .into_pointer_value();

        // 5. vtable_ptr[slot] → fn_ptr.
        let slot_idx = i64_ty.const_int(slot as u64, false);
        let fn_ptr_addr = unsafe {
            self.cg
                .builder
                .build_gep(ptr_ty, vtable_ptr, &[slot_idx], "fn_ptr_addr")
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_method_call",
                    detail: e.to_string(),
                })?
        };
        let fn_ptr = self
            .cg
            .builder
            .build_load(ptr_ty, fn_ptr_addr, "fn_ptr")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_method_call",
                detail: e.to_string(),
            })?
            .into_pointer_value();

        // 6. Build argument list (receiver + remaining args).
        // Args are dispatched the same way `emit_call` does: a Ref-typed
        // arg is the address of a CP variable (VAR/OUT mode, or an
        // open-array fat-pointer's data slot) and resolves through
        // `resolve_pointer`; everything else is a value.
        let mut llvm_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        llvm_args.push(obj_ptr.into());
        for arg in args {
            let val = match arg.ty() {
                IrType::Ref(_) => self.resolve_pointer(arg, value_map)?.into(),
                _ => self.resolve_basic_value(arg, value_map)?,
            };
            llvm_args.push(val.into());
        }

        // Build fn type: all params are `ptr` (opaque pointers), return type from ret_ty.
        let type_lowerer = crate::types::TypeLowerer::new(self.cg.context);
        let ptr_ty_meta = BasicMetadataTypeEnum::from(
            self.cg.context.ptr_type(inkwell::AddressSpace::default()),
        );
        let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> = vec![ptr_ty_meta]; // receiver
        for arg in args {
            let bt = type_lowerer
                .lower_basic(&arg.ty(), Some(&self.cg.planner.named_struct_types))
                .unwrap_or_else(|_| self.cg.context.i64_type().into());
            param_types.push(bt.into());
        }
        let fn_ty = if *ret_ty == newcp_ir::IrType::Void {
            self.cg.context.void_type().fn_type(&param_types, false)
        } else {
            let bt = type_lowerer.lower_basic(ret_ty, Some(&self.cg.planner.named_struct_types))
                .unwrap_or_else(|_| self.cg.context.i64_type().into());
            bt.fn_type(&param_types, false)
        };

        let call = self
            .cg
            .builder
            .build_indirect_call(fn_ty, fn_ptr, &llvm_args, "method_ret")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_method_call",
                detail: e.to_string(),
            })?;

        if let Some(t) = dst {
            if *ret_ty != newcp_ir::IrType::Void {
                let bv = call.try_as_basic_value().unwrap_basic();
                value_map.temp_values.insert(t, bv);
            }
        }
        Ok(())
    }

    fn emit_terminator(
        &mut self,
        term: &newcp_ir::Terminator,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        use newcp_ir::Terminator;
        match term {
            Terminator::Ret { value } => {
                let ret_value = self.resolve_basic_value(value, value_map)?;
                self.cg.builder.build_return(Some(&ret_value)).map_err(|e| {
                    CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: e.to_string(),
                    }
                })?;
            }
            Terminator::RetVoid => {
                self.cg.builder.build_return(None).map_err(|e| CodegenError::Unsupported {
                    stage: "emit_terminator",
                    detail: e.to_string(),
                })?;
            }
            Terminator::Br { target } => {
                let target_block =
                    value_map.block_map.get(target).copied().ok_or_else(|| {
                        CodegenError::Unsupported {
                            stage: "emit_terminator",
                            detail: format!("branch target {} not found", target.render()),
                        }
                    })?;
                self.cg
                    .builder
                    .build_unconditional_branch(target_block)
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: e.to_string(),
                    })?;
            }
                Terminator::CondBr {
                    cond,
                    true_target,
                    false_target,
                } => {
                    let cond_value = self.resolve_basic_value(cond, value_map)?.into_int_value();
                    let true_block = value_map
                        .block_map
                        .get(true_target)
                        .copied()
                        .ok_or_else(|| CodegenError::Unsupported {
                            stage: "emit_terminator",
                            detail: format!("branch target {} not found", true_target.render()),
                        })?;
                    let false_block = value_map
                        .block_map
                        .get(false_target)
                        .copied()
                        .ok_or_else(|| CodegenError::Unsupported {
                            stage: "emit_terminator",
                            detail: format!("branch target {} not found", false_target.render()),
                        })?;
                    self.cg
                        .builder
                        .build_conditional_branch(cond_value, true_block, false_block)
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_terminator",
                            detail: e.to_string(),
                        })?;
                }
            Terminator::Trap { kind } => {
                // Map TrapKind to a stable integer code per the design doc ABI.
                let trap_code: i32 = match kind {
                    newcp_ir::TrapKind::Assert           => 1,
                    newcp_ir::TrapKind::Halt(code)       => *code,
                    newcp_ir::TrapKind::NilDeref         => 2,
                    newcp_ir::TrapKind::ArrayBounds      => 3,
                    newcp_ir::TrapKind::TypeGuard        => 4,
                    newcp_ir::TrapKind::CaseFallthrough  => 5,
                };
                let trap_fn = self.get_or_declare_newcp_trap();
                let code_val = self.cg.context.i32_type().const_int(trap_code as u64, true);
                self.cg
                    .builder
                    .build_call(trap_fn, &[code_val.into()], "trap")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: e.to_string(),
                    })?;
                // After the noreturn call, we still need a terminator.
                self.cg.builder.build_unreachable().map_err(|e| CodegenError::Unsupported {
                    stage: "emit_terminator",
                    detail: e.to_string(),
                })?;
            }
            Terminator::TypeTest { value, ty, true_target, false_target } => {
                let cond = self.emit_type_check(value, ty, value_map)?.into_int_value();
                let true_block = value_map.block_map.get(true_target).copied().ok_or_else(|| {
                    CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: format!("typetest true target {} not found", true_target.render()),
                    }
                })?;
                let false_block = value_map.block_map.get(false_target).copied().ok_or_else(|| {
                    CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: format!("typetest false target {} not found", false_target.render()),
                    }
                })?;
                self.cg
                    .builder
                    .build_conditional_branch(cond, true_block, false_block)
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: e.to_string(),
                    })?;
            }
        }
        Ok(())
    }

    fn get_or_declare_newcp_trap(&self) -> inkwell::values::FunctionValue<'ctx> {
        const NAME: &str = "__newcp_trap";
        if let Some(f) = self.cg.module.get_function(NAME) {
            return f;
        }
        let void_ty = self.cg.context.void_type();
        let i32_ty = self.cg.context.i32_type();
        let fn_ty = void_ty.fn_type(&[i32_ty.into()], false);
        let f = self.cg.module.add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External));
        // Mark as noreturn so LLVM knows the unreachable after it is correct.
        f.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            self.cg.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("noreturn"),
                0,
            ),
        );
        f
    }

    fn bind_proc_slots(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        proc: &IrProcedure,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        // Position once at the entry block's tail; all param stores go here.
        // (ensure_named_slot allocates via its own builder instance.)
        self.cg.builder.position_at_end(value_map.block_map[&proc.entry]);

        for (index, (name, ty)) in proc.params.iter().enumerate() {
            let slot = self.ensure_named_slot(name, ty, value_map)?;
            let param = fn_val.get_nth_param(index as u32).ok_or_else(|| CodegenError::Unsupported {
                stage: "emit",
                detail: format!("missing LLVM parameter {index} for '{}'", proc.name),
            })?;
            if matches!(ty, IrType::Ref(_)) {
                value_map.ref_param_slots.insert(name.clone(), slot);
            }

            // Decide ABI shape for this param:
            //
            // - Aggregate IR types (Array{...}, Named-record, UntaggedRecord)
            //   that are NOT Ref-wrapped are CP value-mode aggregate
            //   parameters.  `declare_procedure` lowers them to `ptr` at
            //   the LLVM signature level so the call site (which always
            //   passes a `designator_addr`) lines up with the formal.
            //   The slot itself is still alloca'd as the value type
            //   (struct or [N x T]); we memmove the caller's bytes into
            //   the slot here so the body's field/index access reads
            //   from a private copy.
            //
            // - Everything else (scalar value, pointer, Ref) is stored
            //   directly into the slot as today.
            let llvm_ty = self.lower_basic_type(ty)?;
            let is_value_aggregate_param = !matches!(ty, IrType::Ref(_))
                && matches!(
                    llvm_ty,
                    BasicTypeEnum::StructType(_) | BasicTypeEnum::ArrayType(_)
                );
            if is_value_aggregate_param {
                self.copy_value_aggregate_param_to_slot(name, llvm_ty, slot, param)?;
            } else {
                self.cg.builder.build_store(slot, param).map_err(|e| CodegenError::Unsupported {
                    stage: "emit",
                    detail: e.to_string(),
                })?;
            }

            // CP §8.1: a value-mode open-array parameter is a private
            // copy.  At the C ABI boundary the caller passes a (data
            // ptr, len) pair, so without intervention the callee's slot
            // aliases the caller's buffer and writes leak back.  When
            // we see (Ptr/UntaggedPtr) + a sibling `<name>$len` formal
            // and ty is *not* Ref-wrapped (i.e. value mode, not
            // IN/VAR/OUT), allocate a stack-local copy of the caller's
            // data, memmove into it, and rebind the slot so subsequent
            // designator accesses go through the local copy.
            let next = proc.params.get(index + 1);
            let is_value_open_array_param = matches!(ty, IrType::Ptr(_) | IrType::UntaggedPtr(_))
                && next.is_some_and(|(next_name, next_ty)| {
                    next_name == &format!("{name}{OPEN_ARRAY_LEN_SUFFIX}")
                        && matches!(next_ty, IrType::I64)
                });
            if is_value_open_array_param {
                let elem_ty = match ty {
                    IrType::Ptr(inner) | IrType::UntaggedPtr(inner) => inner.as_ref().clone(),
                    _ => unreachable!(),
                };
                self.copy_open_array_param_to_local(
                    name,
                    &elem_ty,
                    slot,
                    param,
                    fn_val,
                    index + 1,
                )?;
            }
        }

        if proc.ret_ty != IrType::Void {
            let _ = self.ensure_named_slot("$result", &proc.ret_ty, value_map)?;
        }

        Ok(())
    }

    fn ensure_named_slot(
        &mut self,
        name: &str,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        if let Some(slot) = value_map.named_slots.get(name).copied() {
            return Ok(slot);
        }

        let llvm_ty = self.lower_basic_type(ty)?;
        let entry_block = value_map.entry_block.ok_or_else(|| CodegenError::Unsupported {
            stage: "emit",
            detail: "no entry block recorded for slot allocation".to_string(),
        })?;
        let slot_builder = self.cg.context.create_builder();
        if let Some(first_instr) = entry_block.get_first_instruction() {
            slot_builder.position_before(&first_instr);
        } else {
            slot_builder.position_at_end(entry_block);
        }

        // For record-typed locals, mirror the heap layout
        // (`__newcp_new_rec`) so `__newcp_type_test` works uniformly:
        // alloca a `[16-byte BlockHeader] [record payload]` buffer
        // and hand back a pointer to the payload.  The header's tag
        // field is initialised at proc entry with the static
        // TypeDesc — local TypeDescs get a constant write, cross-
        // module ones go through the `__newcp_lookup_typedesc`
        // registry so the address resolves once imports are loaded.
        if let Some(record_name) = self.tagged_record_name_for_ir_type(ty) {
            return self.alloca_record_with_header(
                name,
                ty,
                &record_name,
                value_map,
                slot_builder,
            );
        }

        let slot = slot_builder.build_alloca(llvm_ty, name).map_err(|e| CodegenError::Unsupported {
            stage: "emit",
            detail: e.to_string(),
        })?;
        value_map.named_slots.insert(name.to_string(), slot);
        Ok(slot)
    }

    /// Return the simple/qualified record-type name for `ty` if it
    /// resolves to a record type that has a TypeDesc emitted (locally
    /// or cross-module).  Pointer / non-record / unknown types return
    /// `None`.
    fn tagged_record_name_for_ir_type(&self, ty: &IrType) -> Option<String> {
        match ty {
            IrType::Named(name) => {
                // Record types live in `named_struct_types` after the
                // planner has walked the module.  Pointer aliases land
                // there too, so distinguish by checking whether the
                // basic-type lower for `Named(name)` is a struct.
                let is_struct = self
                    .cg
                    .planner
                    .named_struct_types
                    .get(name.as_str())
                    .is_some();
                if is_struct {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Alloca a `[16-byte header][record payload]` buffer, write the
    /// static TypeDesc into the tag slot, and stash the payload
    /// pointer (= alloca + 16) in `named_slots[name]`.
    fn alloca_record_with_header(
        &mut self,
        name: &str,
        record_ty: &IrType,
        record_name: &str,
        value_map: &mut ValueMap<'ctx>,
        slot_builder: inkwell::builder::Builder<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        use inkwell::AddressSpace;
        let i8_ty = self.cg.context.i8_type();
        let i64_ty = self.cg.context.i64_type();
        let ptr_ty = self.cg.context.ptr_type(AddressSpace::default());

        // 1. Resolve record size.
        let record_llvm_ty = self.lower_basic_type(record_ty)?;
        let record_size_val = match record_llvm_ty {
            inkwell::types::BasicTypeEnum::StructType(t) => t.size_of(),
            other => {
                return Err(CodegenError::Unsupported {
                    stage: "alloca_record_with_header",
                    detail: format!("expected struct type for record alloca, got {other:?}"),
                });
            }
        }
        .ok_or_else(|| CodegenError::Unsupported {
            stage: "alloca_record_with_header",
            detail: format!("struct {record_name} has no computable size"),
        })?;

        // 2. Total = header(16) + record_size.  Round up to 8-byte
        // alignment so the payload is naturally aligned.
        let header_size = i64_ty.const_int(16, false);
        let total_bytes = slot_builder
            .build_int_add(record_size_val, header_size, "rec.slot.size")
            .map_err(|e| CodegenError::Unsupported {
                stage: "alloca_record_with_header",
                detail: e.to_string(),
            })?;
        let raw_alloca = slot_builder
            .build_array_alloca(i8_ty, total_bytes, &format!("{name}.shadow"))
            .map_err(|e| CodegenError::Unsupported {
                stage: "alloca_record_with_header",
                detail: e.to_string(),
            })?;
        // 3. Payload = raw_alloca + 16.
        let payload_offset = i64_ty.const_int(16, false);
        let payload_ptr = unsafe {
            slot_builder
                .build_in_bounds_gep(i8_ty, raw_alloca, &[payload_offset], name)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "alloca_record_with_header",
                    detail: e.to_string(),
                })?
        };

        // 4. Resolve the TypeDesc address and write it into the tag
        // field at offset 0.  Local TypeDescs are constant globals;
        // cross-module ones get a runtime lookup.
        let typedesc_ptr_val: inkwell::values::PointerValue<'ctx> =
            if let Some(g) = self.cg.planner.type_desc_globals.get(record_name) {
                g.as_pointer_value()
            } else {
                // Cross-module TypeDesc — call __newcp_lookup_typedesc
                // at proc entry.  This requires the imported module's
                // __init_types to have run already; the loader's
                // dependency-ordered init guarantees that.
                let lookup_fn = self.get_or_declare_lookup_typedesc();
                let qualified = if record_name.contains('.') {
                    record_name.to_string()
                } else {
                    record_name.to_string()
                };
                let name_global = self.cg.get_or_emit_cstring_constant(&qualified);
                let call = slot_builder
                    .build_call(lookup_fn, &[name_global.into()], "td.local.lookup")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "alloca_record_with_header",
                        detail: e.to_string(),
                    })?;
                let td_i64 = call.try_as_basic_value().unwrap_basic().into_int_value();
                slot_builder
                    .build_int_to_ptr(td_i64, ptr_ty, "td.local.ptr")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "alloca_record_with_header",
                        detail: e.to_string(),
                    })?
            };
        // tag is at raw_alloca + 0.
        let _ = slot_builder
            .build_store(raw_alloca, typedesc_ptr_val)
            .map_err(|e| CodegenError::Unsupported {
                stage: "alloca_record_with_header",
                detail: e.to_string(),
            })?;

        value_map.named_slots.insert(name.to_string(), payload_ptr);
        Ok(payload_ptr)
    }

    fn lower_basic_type(&self, ty: &IrType) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        self.cg
            .lowerer
            .lower_basic(ty, Some(&self.cg.planner.named_struct_types))
    }

    fn resolve_pointer(
        &mut self,
        value: &IrValue,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        match value {
            IrValue::GlobalRef(name, IrType::Ref(inner)) => {
                if let Some(slot) = value_map.named_slots.get(name).copied() {
                    if let Some(ref_slot) = value_map.ref_param_slots.get(name).copied() {
                        self.cg
                            .builder
                            .build_load(
                                self.cg.context.ptr_type(inkwell::AddressSpace::default()),
                                ref_slot,
                                &format!("{name}_ref"),
                            )
                            .map(|value| value.into_pointer_value())
                            .map_err(|e| CodegenError::Unsupported {
                                stage: "emit",
                                detail: e.to_string(),
                            })
                    } else {
                        Ok(slot)
                    }
                } else if let Some(global) = self.cg.planner.globals.get(name).copied() {
                    Ok(global)
                } else {
                    self.ensure_named_slot(name, inner, value_map)
                }
            }
            IrValue::ImportRef(module, name, IrType::Ref(_inner)) => {
                let slot = self.get_or_declare_imported_global_slot(module, name);
                self.cg
                    .builder
                    .build_load(
                        self.cg.context.ptr_type(AddressSpace::default()),
                        slot,
                        &format!("{module}_{name}_addr"),
                    )
                    .map(|value| value.into_pointer_value())
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit",
                        detail: e.to_string(),
                    })
            }
            IrValue::Temp(id, _) => value_map
                .temp_values
                .get(id)
                .copied()
                .ok_or_else(|| CodegenError::Unsupported {
                    stage: "emit",
                    detail: format!("temporary {} not found", id.render()),
                })?
                .into_pointer_value()
                .try_into()
                .map_err(|_| CodegenError::Unsupported {
                    stage: "emit",
                    detail: format!("temporary {} is not a pointer", id.render()),
                }),
            other => Err(CodegenError::Unsupported {
                stage: "emit",
                detail: format!("unsupported address operand {other:?}"),
            }),
        }
    }

    fn resolve_basic_value(
        &mut self,
        value: &IrValue,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match value {
            IrValue::Temp(id, _) => value_map
                .temp_values
                .get(id)
                .copied()
                .ok_or_else(|| CodegenError::Unsupported {
                    stage: "emit",
                    detail: format!("temporary {} not found", id.render()),
                }),
            IrValue::ConstInt(v, ty) => Ok(self.const_int(*v, ty).into()),
            IrValue::ConstReal(v, IrType::F32) => Ok(self.cg.context.f32_type().const_float(*v).into()),
            IrValue::ConstReal(v, _) => Ok(self.cg.context.f64_type().const_float(*v).into()),
            IrValue::ConstBool(v) => Ok(self.cg.context.bool_type().const_int(u64::from(*v), false).into()),
            IrValue::ConstChar(v) => Ok(self.cg.context.i32_type().const_int(*v as u64, false).into()),
            IrValue::ConstStr(s, elem_ty) => {
                let ptr = self.cg.get_or_emit_string_constant(s, elem_ty);
                Ok(ptr.into())
            }
            IrValue::Null(_) => Ok(self.cg.context.ptr_type(inkwell::AddressSpace::default()).const_null().into()),
            IrValue::GlobalRef(name, _) => {
                // A bare GlobalRef here means a first-class procedure value
                // (e.g. `fn := ReturnSeven`).  Return the function pointer.
                if let Some(&fn_val) = self.cg.planner.functions.get(name.as_str()) {
                    Ok(fn_val.as_global_value().as_pointer_value().into())
                } else {
                    Err(CodegenError::Unsupported {
                        stage: "emit",
                        detail: format!("unsupported value operand GlobalRef({name})"),
                    })
                }
            }
            other => Err(CodegenError::Unsupported {
                stage: "emit",
                detail: format!("unsupported value operand {other:?}"),
            }),
        }
    }

    fn emit_binop(
        &mut self,
        op: BinOp,
        left: &IrValue,
        right: &IrValue,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let left_value = self.resolve_basic_value(left, value_map)?;
        let right_value = self.resolve_basic_value(right, value_map)?;

        if matches!(left.ty(), IrType::F32 | IrType::F64) {
            let lhs = left_value.into_float_value();
            let rhs = right_value.into_float_value();
            let value = match op {
                BinOp::Add => self.cg.builder.build_float_add(lhs, rhs, "fadd").map(BasicValueEnum::from),
                BinOp::Sub => self.cg.builder.build_float_sub(lhs, rhs, "fsub").map(BasicValueEnum::from),
                BinOp::Mul => self.cg.builder.build_float_mul(lhs, rhs, "fmul").map(BasicValueEnum::from),
                BinOp::Div => self.cg.builder.build_float_div(lhs, rhs, "fdiv").map(BasicValueEnum::from),
                BinOp::Eq => self.cg.builder.build_float_compare(FloatPredicate::OEQ, lhs, rhs, "feq").map(BasicValueEnum::from),
                BinOp::Ne => self.cg.builder.build_float_compare(FloatPredicate::ONE, lhs, rhs, "fne").map(BasicValueEnum::from),
                BinOp::Lt => self.cg.builder.build_float_compare(FloatPredicate::OLT, lhs, rhs, "flt").map(BasicValueEnum::from),
                BinOp::Le => self.cg.builder.build_float_compare(FloatPredicate::OLE, lhs, rhs, "fle").map(BasicValueEnum::from),
                BinOp::Gt => self.cg.builder.build_float_compare(FloatPredicate::OGT, lhs, rhs, "fgt").map(BasicValueEnum::from),
                BinOp::Ge => self.cg.builder.build_float_compare(FloatPredicate::OGE, lhs, rhs, "fge").map(BasicValueEnum::from),
                other => {
                    return Err(CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: format!("unsupported float binop {other:?}"),
                    });
                }
            }
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
            return Ok(value.into());
        }

        // Pointer equality / inequality: `ptr == null`, `ptr != null`, `ptr == ptr`.
        // In opaque-pointer LLVM, use `icmp eq/ne` directly on pointer values.
        if left_value.is_pointer_value() || right_value.is_pointer_value() {
            let pred = match op {
                BinOp::Eq => IntPredicate::EQ,
                BinOp::Ne => IntPredicate::NE,
                _ => {
                    return Err(CodegenError::Unsupported {
                        stage: "emit_binop",
                        detail: format!("non-equality pointer comparison {op:?}"),
                    });
                }
            };
            let null_ptr = self.cg.context.ptr_type(inkwell::AddressSpace::default()).const_null();
            let lhs_ptr = if left_value.is_pointer_value() { left_value.into_pointer_value() } else { null_ptr };
            let rhs_ptr = if right_value.is_pointer_value() { right_value.into_pointer_value() } else { null_ptr };
            return self
                .cg
                .builder
                .build_int_compare(pred, lhs_ptr, rhs_ptr, "pcmp")
                .map(BasicValueEnum::from)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_binop",
                    detail: e.to_string(),
                });
        }

        let lhs = left_value.into_int_value();
        let rhs = right_value.into_int_value();

        // LLVM ICmp requires both operands to have the same type.
        // Widen the narrower operand to match the wider one.
        let (lhs, rhs) = {
            let lbits = lhs.get_type().get_bit_width();
            let rbits = rhs.get_type().get_bit_width();
            if lbits < rbits {
                let wider = if self.is_unsigned_type(&left.ty()) {
                    self.cg.builder.build_int_z_extend(lhs, rhs.get_type(), "cmp.zext")
                } else {
                    self.cg.builder.build_int_s_extend(lhs, rhs.get_type(), "cmp.sext")
                }.map_err(|e| CodegenError::Unsupported { stage: "emit_binop", detail: e.to_string() })?;
                (wider, rhs)
            } else if rbits < lbits {
                let wider = if self.is_unsigned_type(&right.ty()) {
                    self.cg.builder.build_int_z_extend(rhs, lhs.get_type(), "cmp.zext")
                } else {
                    self.cg.builder.build_int_s_extend(rhs, lhs.get_type(), "cmp.sext")
                }.map_err(|e| CodegenError::Unsupported { stage: "emit_binop", detail: e.to_string() })?;
                (lhs, wider)
            } else {
                (lhs, rhs)
            }
        };
        let predicate = match op {
            BinOp::Eq => Some(IntPredicate::EQ),
            BinOp::Ne => Some(IntPredicate::NE),
            BinOp::Lt => Some(self.int_predicate(left, IntPredicate::SLT, IntPredicate::ULT)),
            BinOp::Le => Some(self.int_predicate(left, IntPredicate::SLE, IntPredicate::ULE)),
            BinOp::Gt => Some(self.int_predicate(left, IntPredicate::SGT, IntPredicate::UGT)),
            BinOp::Ge => Some(self.int_predicate(left, IntPredicate::SGE, IntPredicate::UGE)),
            _ => None,
        };

        let value = if let Some(pred) = predicate {
            self.cg.builder.build_int_compare(pred, lhs, rhs, "icmp").map(|v| v.into())
        } else {
            match op {
                BinOp::Add => self.cg.builder.build_int_add(lhs, rhs, "iadd").map(|v| v.into()),
                BinOp::Sub => self.cg.builder.build_int_sub(lhs, rhs, "isub").map(|v| v.into()),
                BinOp::Mul => self.cg.builder.build_int_mul(lhs, rhs, "imul").map(|v| v.into()),
                BinOp::Div => {
                    let pred = self.int_predicate(left, IntPredicate::SLT, IntPredicate::ULT);
                    if matches!(pred, IntPredicate::ULT) {
                        self.cg.builder.build_int_unsigned_div(lhs, rhs, "udiv").map(|v| v.into())
                    } else {
                        self.cg.builder.build_int_signed_div(lhs, rhs, "sdiv").map(|v| v.into())
                    }
                }
                BinOp::Mod => {
                    let pred = self.int_predicate(left, IntPredicate::SLT, IntPredicate::ULT);
                    if matches!(pred, IntPredicate::ULT) {
                        self.cg.builder.build_int_unsigned_rem(lhs, rhs, "urem").map(|v| v.into())
                    } else {
                        self.cg.builder.build_int_signed_rem(lhs, rhs, "srem").map(|v| v.into())
                    }
                }
                BinOp::And => self.cg.builder.build_and(lhs, rhs, "and").map(|v| v.into()),
                BinOp::Or => self.cg.builder.build_or(lhs, rhs, "or").map(|v| v.into()),
                BinOp::Xor => self.cg.builder.build_xor(lhs, rhs, "xor").map(|v| v.into()),
                BinOp::Shl => self.cg.builder.build_left_shift(lhs, rhs, "shl").map(|v| v.into()),
                BinOp::Shr => self.cg.builder.build_right_shift(
                    lhs,
                    rhs,
                    !self.is_unsigned_type(&left.ty()),
                    "shr",
                ).map(|v| v.into()),
                BinOp::In => {
                    // x IN s: test whether bit x of set s is set.
                    // result = (s >> x) & 1 != 0
                    // `return` diverges this arm's type, bypassing the outer `.map_err`.
                    let x_cast = self.cast_shift_to_width(lhs, rhs.get_type(), "in.cast")?;
                    let shifted = self
                        .cg
                        .builder
                        .build_right_shift(rhs, x_cast, false, "in.shr")
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    let one = rhs.get_type().const_int(1, false);
                    let masked = self
                        .cg
                        .builder
                        .build_and(shifted, one, "in.and")
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })?;
                    return self
                        .cg
                        .builder
                        .build_int_compare(
                            IntPredicate::NE,
                            masked,
                            rhs.get_type().const_zero(),
                            "in.ne",
                        )
                        .map(|v| BasicValueEnum::from(v))
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        });
                }
                // Eq/Ne/Lt/Le/Gt/Ge are handled by the predicate path above.
                _ => unreachable!("binop {op:?} should have been routed through the predicate path"),
            }
        }
        .map_err(|e| CodegenError::Unsupported {
            stage: "emit_instr",
            detail: e.to_string(),
        })?;

        let _ = ty;
        Ok(value)
    }

    fn emit_unop(
        &mut self,
        op: UnOp,
        operand: &IrValue,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let operand_value = self.resolve_basic_value(operand, value_map)?;

        match op {
            UnOp::Neg if matches!(operand.ty(), IrType::F32 | IrType::F64) => self
                .cg
                .builder
                .build_float_neg(operand_value.into_float_value(), "fneg")
                .map(Into::into)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
            UnOp::Neg => self
                .cg
                .builder
                .build_int_neg(operand_value.into_int_value(), "neg")
                .map(Into::into)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
            UnOp::Not => self
                .cg
                .builder
                .build_not(operand_value.into_int_value(), "not")
                .map(Into::into)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
            UnOp::BitNot => self
                .cg
                .builder
                .build_not(operand_value.into_int_value(), "bitnot")
                .map(Into::into)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
        }
        .map(|value| {
            let _ = ty;
            value
        })
    }

    fn resolve_raw_pointer(
        &mut self,
        addr: &IrValue,
        pointee_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        let addr_value = self.resolve_basic_value(addr, value_map)?.into_int_value();
        let _ = self.lower_basic_type(pointee_ty)?;
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        self.cg
            .builder
            .build_int_to_ptr(addr_value, ptr_ty, "rawptr")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })
    }

    fn emit_lsh(
        &mut self,
        value: &IrValue,
        shift: &IrValue,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let value_int = self.resolve_basic_value(value, value_map)?.into_int_value();
        let shift_int = self.resolve_basic_value(shift, value_map)?.into_int_value();
        let value_ty = value_int.get_type();
        let (is_negative, left_shift, right_shift) =
            self.signed_shift_operands(shift_int, value_ty, "lsh")?;
        let shl = self
            .cg
            .builder
            .build_left_shift(value_int, left_shift, "lsh.left.value")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let lshr = self
            .cg
            .builder
            .build_right_shift(value_int, right_shift, false, "lsh.right.value")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let selected = self
            .cg
            .builder
            .build_select(is_negative, lshr, shl, "lsh.result")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let _ = ty;
        Ok(selected)
    }

    fn emit_ash(
        &mut self,
        value: &IrValue,
        shift: &IrValue,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let value_int = self.resolve_basic_value(value, value_map)?.into_int_value();
        let shift_int = self.resolve_basic_value(shift, value_map)?.into_int_value();
        let value_ty = value_int.get_type();
        let (is_negative, left_shift, right_shift) =
            self.signed_shift_operands(shift_int, value_ty, "ash")?;
        let shl = self
            .cg
            .builder
            .build_left_shift(value_int, left_shift, "ash.left.value")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        // Arithmetic right shift (sign-extends).
        let ashr = self
            .cg
            .builder
            .build_right_shift(value_int, right_shift, true, "ash.right.value")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let selected = self
            .cg
            .builder
            .build_select(is_negative, ashr, shl, "ash.result")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let _ = ty;
        Ok(selected)
    }

    fn emit_bitcast(
        &mut self,
        value: &IrValue,
        dest_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let source = self.resolve_basic_value(value, value_map)?;
        let source_ty = value.ty();

        if source_ty == *dest_ty {
            return Ok(source);
        }

        let dest_basic = self.lower_basic_type(dest_ty)?;
        match (source, dest_basic) {
            (BasicValueEnum::IntValue(int_value), BasicTypeEnum::IntType(dest_int_ty)) => {
                if int_value.get_type() == dest_int_ty {
                    Ok(int_value.into())
                } else {
                    self.cg
                        .builder
                        .build_int_cast(int_value, dest_int_ty, "bitcast.int")
                        .map(Into::into)
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_instr",
                            detail: e.to_string(),
                        })
                }
            }
            (BasicValueEnum::PointerValue(ptr_value), BasicTypeEnum::PointerType(dest_ptr_ty)) => {
                self.cg
                    .builder
                    .build_pointer_cast(ptr_value, dest_ptr_ty, "bitcast.ptr")
                    .map(Into::into)
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: e.to_string(),
                    })
            }
            (BasicValueEnum::PointerValue(ptr_value), BasicTypeEnum::IntType(dest_int_ty)) => self
                .cg
                .builder
                .build_ptr_to_int(ptr_value, dest_int_ty, "bitcast.ptrtoint")
                .map(Into::into)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
            (BasicValueEnum::IntValue(int_value), BasicTypeEnum::PointerType(dest_ptr_ty)) => self
                .cg
                .builder
                .build_int_to_ptr(int_value, dest_ptr_ty, "bitcast.inttoptr")
                .map(Into::into)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
            (basic_value, dest_basic_ty) => self
                .cg
                .builder
                .build_bit_cast(basic_value, dest_basic_ty, "bitcast")
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                }),
        }
    }

    /// Emit a numeric/char type-coercion cast (Instr::Cast).
    ///
    /// Rules (by source → destination type):
    /// - int → wider int (signed src): `sext`
    /// - int → wider int (unsigned/char src): `zext`
    /// - int → narrower int / char: `trunc`
    /// - float → float: `fpext` or `fptrunc`
    /// - float → int: `fptosi`
    /// - int → float: `sitofp` / `uitofp`
    /// - same type: identity (e.g. ABS of same float type uses llvm.fabs intrinsic)
    fn emit_cast(
        &mut self,
        value: &IrValue,
        to_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let from_ty = value.ty();
        let src = self.resolve_basic_value(value, value_map)?;

        // Float absolute value: same-type Cast on floats
        if from_ty == *to_ty {
            match src {
                BasicValueEnum::FloatValue(fv) => {
                    let intrinsic_name = if matches!(to_ty, IrType::F32) { "llvm.fabs.f32" } else { "llvm.fabs.f64" };
                    let fabs_ty = fv.get_type().fn_type(&[fv.get_type().into()], false);
                    let fabs = self.cg.module.get_function(intrinsic_name).unwrap_or_else(|| {
                        self.cg.module.add_function(intrinsic_name, fabs_ty, None)
                    });
                    let result = self.cg.builder.build_call(fabs, &[fv.into()], "fabs")
                        .map_err(|e| CodegenError::Unsupported { stage: "emit_cast", detail: e.to_string() })?;
                    return Ok(result.try_as_basic_value().unwrap_basic());
                }
                _ => return Ok(src),
            }
        }

        let dest_basic = self.lower_basic_type(to_ty)?;
        match (src, dest_basic) {
            // Integer → integer
            (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(dest_int)) => {
                let src_bits = iv.get_type().get_bit_width();
                let dst_bits = dest_int.get_bit_width();
                let result = if dst_bits > src_bits {
                    // widen: use zext for unsigned/char types, sext for signed
                    if self.is_unsigned_type(&from_ty) {
                        self.cg.builder.build_int_z_extend(iv, dest_int, "cast.zext")
                    } else {
                        self.cg.builder.build_int_s_extend(iv, dest_int, "cast.sext")
                    }
                } else {
                    self.cg.builder.build_int_truncate(iv, dest_int, "cast.trunc")
                };
                result.map(Into::into).map_err(|e| CodegenError::Unsupported { stage: "emit_cast", detail: e.to_string() })
            }
            // Float → float
            (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(dest_float)) => {
                let src_bits = fv.get_type().get_bit_width();
                let dst_bits = dest_float.get_bit_width();
                let result = if dst_bits > src_bits {
                    self.cg.builder.build_float_ext(fv, dest_float, "cast.fpext")
                } else {
                    self.cg.builder.build_float_trunc(fv, dest_float, "cast.fptrunc")
                };
                result.map(Into::into).map_err(|e| CodegenError::Unsupported { stage: "emit_cast", detail: e.to_string() })
            }
            // Float → int
            (BasicValueEnum::FloatValue(fv), BasicTypeEnum::IntType(dest_int)) => {
                self.cg.builder.build_float_to_signed_int(fv, dest_int, "cast.ftoi")
                    .map(Into::into)
                    .map_err(|e| CodegenError::Unsupported { stage: "emit_cast", detail: e.to_string() })
            }
            // Int → float
            (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(dest_float)) => {
                if self.is_unsigned_type(&from_ty) {
                    self.cg.builder.build_unsigned_int_to_float(iv, dest_float, "cast.uitof")
                } else {
                    self.cg.builder.build_signed_int_to_float(iv, dest_float, "cast.sitof")
                }.map(Into::into).map_err(|e| CodegenError::Unsupported { stage: "emit_cast", detail: e.to_string() })
            }
            // Pointer → pointer: all pointers are opaque `ptr` in LLVM, so this is
            // a pass-through.  Arises e.g. when casting a function-pointer value to a
            // named procedure-type alias (TYPE Handler = PROCEDURE(...)).
            (BasicValueEnum::PointerValue(pv), BasicTypeEnum::PointerType(_)) => Ok(pv.into()),
            (sv, _) => Err(CodegenError::Unsupported {
                stage: "emit_cast",
                detail: format!("unsupported cast from {:?} to {}", sv.get_type(), to_ty.render()),
            }),
        }
    }


    fn emit_rot(
        &mut self,
        value: &IrValue,
        shift: &IrValue,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let value_int = self.resolve_basic_value(value, value_map)?.into_int_value();
        let shift_int = self.resolve_basic_value(shift, value_map)?.into_int_value();
        let value_ty = value_int.get_type();
        let bit_width = value_ty.get_bit_width() as u64;
        let width_value = value_ty.const_int(bit_width, false);
        let (is_negative, left_shift, right_shift) =
            self.signed_shift_operands(shift_int, value_ty, "rot")?;
        let left_amount = self
            .cg
            .builder
            .build_int_unsigned_rem(left_shift, width_value, "rot.left.amount")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let right_amount = self
            .cg
            .builder
            .build_int_unsigned_rem(right_shift, width_value, "rot.right.amount")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let rotl = self.build_rotate_call(true, value_int, left_amount)?;
        let rotr = self.build_rotate_call(false, value_int, right_amount)?;
        let selected = self
            .cg
            .builder
            .build_select(is_negative, rotr, rotl, "rot.result")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let _ = ty;
        Ok(selected)
    }

    fn cast_shift_to_width(
        &self,
        shift: inkwell::values::IntValue<'ctx>,
        target_ty: inkwell::types::IntType<'ctx>,
        name: &str,
    ) -> Result<inkwell::values::IntValue<'ctx>, CodegenError> {
        if shift.get_type() == target_ty {
            return Ok(shift);
        }

        self.cg
            .builder
            .build_int_cast(shift, target_ty, name)
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })
    }

    /// Compute direction-dependent shift operands used by both `emit_lsh` and `emit_rot`.
    ///
    /// Returns `(is_negative, left_cast, right_cast)`:
    /// - `is_negative`: `i1` true when `shift < 0` (right-shift direction)
    /// - `left_cast`: `shift` zero-/sign-extended to `value_ty` width
    /// - `right_cast`: `abs(shift)` zero-/sign-extended to `value_ty` width
    fn signed_shift_operands(
        &self,
        shift: inkwell::values::IntValue<'ctx>,
        value_ty: inkwell::types::IntType<'ctx>,
        prefix: &str,
    ) -> Result<
        (
            inkwell::values::IntValue<'ctx>,
            inkwell::values::IntValue<'ctx>,
            inkwell::values::IntValue<'ctx>,
        ),
        CodegenError,
    > {
        let zero = shift.get_type().const_zero();
        let is_negative = self
            .cg
            .builder
            .build_int_compare(IntPredicate::SLT, shift, zero, &format!("{prefix}.neg"))
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let neg_shift = self
            .cg
            .builder
            .build_int_neg(shift, &format!("{prefix}.negated"))
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let left_cast = self.cast_shift_to_width(shift, value_ty, &format!("{prefix}.left"))?;
        let right_cast =
            self.cast_shift_to_width(neg_shift, value_ty, &format!("{prefix}.right"))?;
        Ok((is_negative, left_cast, right_cast))
    }

    fn build_rotate_call(
        &self,
        left: bool,
        value: inkwell::values::IntValue<'ctx>,
        amount: inkwell::values::IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let intrinsic = self.get_or_declare_rotate_intrinsic(value.get_type(), left);
        let call = self
            .cg
            .builder
            .build_call(intrinsic, &[value.into(), value.into(), amount.into()], if left { "rotl" } else { "rotr" })
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        Ok(call.try_as_basic_value().unwrap_basic())
    }

    fn get_or_declare_rotate_intrinsic(
        &self,
        int_ty: inkwell::types::IntType<'ctx>,
        left: bool,
    ) -> FunctionValue<'ctx> {
        let name = if left {
            format!("llvm.fshl.i{}", int_ty.get_bit_width())
        } else {
            format!("llvm.fshr.i{}", int_ty.get_bit_width())
        };
        if let Some(function) = self.cg.module.get_function(&name) {
            return function;
        }

        let fn_ty = int_ty.fn_type(&[int_ty.into(), int_ty.into(), int_ty.into()], false);
        self.cg.module.add_function(&name, fn_ty, None)
    }

    fn emit_memcopy(
        &mut self,
        dst: &IrValue,
        src: &IrValue,
        len: &IrValue,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let i1_ty = self.cg.context.bool_type();
        let dst_addr = self.resolve_basic_value(dst, value_map)?.into_int_value();
        let src_addr = self.resolve_basic_value(src, value_map)?.into_int_value();
        let len_value = self.resolve_basic_value(len, value_map)?.into_int_value();
        let dst_ptr = self
            .cg
            .builder
            .build_int_to_ptr(dst_addr, ptr_ty, "memmove.dst")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let src_ptr = self
            .cg
            .builder
            .build_int_to_ptr(src_addr, ptr_ty, "memmove.src")
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        let len_i64 = self.cast_shift_to_width(len_value, i64_ty, "memmove.len")?;
        let memmove = self.get_or_declare_memmove();
        self.cg
            .builder
            .build_call(
                memmove,
                &[dst_ptr.into(), src_ptr.into(), len_i64.into(), i1_ty.const_zero().into()],
                "memmove",
            )
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;
        Ok(())
    }

    /// Prologue helper for value-mode aggregate params (records and
    /// fixed-size arrays).  The LLVM formal for these is `ptr` (set up
    /// in `declare_procedure`), but the slot is alloca'd as the value
    /// type — copy `sizeof(slot)` bytes from the caller-supplied
    /// pointer into the slot so the body sees a private copy.
    fn copy_value_aggregate_param_to_slot(
        &mut self,
        name: &str,
        slot_llvm_ty: BasicTypeEnum<'ctx>,
        slot: PointerValue<'ctx>,
        incoming_data_ptr: BasicValueEnum<'ctx>,
    ) -> Result<(), CodegenError> {
        let i1_ty = self.cg.context.bool_type();
        let total_bytes = match slot_llvm_ty {
            BasicTypeEnum::StructType(t) => t.size_of().ok_or_else(|| CodegenError::Unsupported {
                stage: "value_aggregate_param_copy",
                detail: format!("struct param '{name}' has no size"),
            })?,
            BasicTypeEnum::ArrayType(t) => t.size_of().ok_or_else(|| CodegenError::Unsupported {
                stage: "value_aggregate_param_copy",
                detail: format!("array param '{name}' has no size"),
            })?,
            _ => unreachable!("caller filtered to struct/array LLVM types"),
        };
        let memmove = self.get_or_declare_memmove();
        self.cg
            .builder
            .build_call(
                memmove,
                &[
                    slot.into(),
                    incoming_data_ptr.into(),
                    total_bytes.into(),
                    i1_ty.const_zero().into(),
                ],
                &format!("{name}.value.memmove"),
            )
            .map_err(|e| CodegenError::Unsupported {
                stage: "value_aggregate_param_copy",
                detail: e.to_string(),
            })?;
        Ok(())
    }

    /// Prologue helper for value-mode open-array params.  Emits a stack
    /// alloca sized at `len * sizeof(elem)`, memmoves the caller's data
    /// into it, and rewrites the param's slot so subsequent designator
    /// accesses go through the private copy.  Caller must have already
    /// stored the incoming data pointer into `slot`; this routine
    /// overwrites that store.
    fn copy_open_array_param_to_local(
        &mut self,
        name: &str,
        elem_ty: &IrType,
        slot: PointerValue<'ctx>,
        incoming_data_ptr: BasicValueEnum<'ctx>,
        fn_val: FunctionValue<'ctx>,
        len_param_index: usize,
    ) -> Result<(), CodegenError> {
        let i64_ty = self.cg.context.i64_type();
        let i8_ty = self.cg.context.i8_type();
        let i1_ty = self.cg.context.bool_type();

        let elem_llvm_ty = self.lower_basic_type(elem_ty)?;
        let elem_size: inkwell::values::IntValue<'ctx> = match elem_llvm_ty {
            inkwell::types::BasicTypeEnum::IntType(t) => t.size_of(),
            inkwell::types::BasicTypeEnum::FloatType(t) => t.size_of(),
            inkwell::types::BasicTypeEnum::PointerType(t) => t.size_of(),
            inkwell::types::BasicTypeEnum::StructType(t) => t.size_of().ok_or_else(|| {
                CodegenError::Unsupported {
                    stage: "open_array_value_copy",
                    detail: format!("struct element {elem_ty:?} has no size"),
                }
            })?,
            inkwell::types::BasicTypeEnum::ArrayType(t) => t.size_of().ok_or_else(|| {
                CodegenError::Unsupported {
                    stage: "open_array_value_copy",
                    detail: format!("array element {elem_ty:?} has no size"),
                }
            })?,
            other => {
                return Err(CodegenError::Unsupported {
                    stage: "open_array_value_copy",
                    detail: format!("element type {other:?} has no computable size"),
                });
            }
        };

        let len_param = fn_val.get_nth_param(len_param_index as u32).ok_or_else(|| {
            CodegenError::Unsupported {
                stage: "open_array_value_copy",
                detail: format!("missing $len param for value-mode open array '{name}'"),
            }
        })?;
        let len_int = len_param.into_int_value();
        let len_i64 = if len_int.get_type().get_bit_width() == 64 {
            len_int
        } else {
            self.cg
                .builder
                .build_int_z_extend(len_int, i64_ty, &format!("{name}.copy.len"))
                .map_err(|e| CodegenError::Unsupported {
                    stage: "open_array_value_copy",
                    detail: e.to_string(),
                })?
        };

        let byte_count = self
            .cg
            .builder
            .build_int_mul(len_i64, elem_size, &format!("{name}.copy.bytes"))
            .map_err(|e| CodegenError::Unsupported {
                stage: "open_array_value_copy",
                detail: e.to_string(),
            })?;

        let local = self
            .cg
            .builder
            .build_array_alloca(i8_ty, byte_count, &format!("{name}.copy"))
            .map_err(|e| CodegenError::Unsupported {
                stage: "open_array_value_copy",
                detail: e.to_string(),
            })?;

        let memmove = self.get_or_declare_memmove();
        self.cg
            .builder
            .build_call(
                memmove,
                &[
                    local.into(),
                    incoming_data_ptr.into(),
                    byte_count.into(),
                    i1_ty.const_zero().into(),
                ],
                &format!("{name}.copy.memmove"),
            )
            .map_err(|e| CodegenError::Unsupported {
                stage: "open_array_value_copy",
                detail: e.to_string(),
            })?;

        self.cg
            .builder
            .build_store(slot, local)
            .map_err(|e| CodegenError::Unsupported {
                stage: "open_array_value_copy",
                detail: e.to_string(),
            })?;

        Ok(())
    }

    fn get_or_declare_memmove(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "llvm.memmove.p0.p0.i64";
        if let Some(function) = self.cg.module.get_function(NAME) {
            return function;
        }

        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let i1_ty = self.cg.context.bool_type();
        let fn_ty = self
            .cg
            .context
            .void_type()
            .fn_type(&[ptr_ty.into(), ptr_ty.into(), i64_ty.into(), i1_ty.into()], false);
        self.cg.module.add_function(NAME, fn_ty, None)
    }

    fn emit_call(
        &mut self,
        dst: Option<TempId>,
        callee: &IrValue,
        args: &[IrValue],
        ret_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        // Resolve arguments first (needed for both direct and indirect calls).
        let call_args: Vec<BasicMetadataValueEnum<'ctx>> = args
            .iter()
            .map(|arg| match arg.ty() {
                IrType::Ref(_) => self.resolve_pointer(arg, value_map).map(Into::into),
                _ => self.resolve_basic_value(arg, value_map).map(Into::into),
            })
            .collect::<Result<_, _>>()?;

        let call_label = dst.map(|id| id.render()).unwrap_or_else(|| "call".to_owned());

        // Indirect call through a procedure-type variable (IrValue::Temp callee).
        if let IrValue::Temp(id, _) = callee {
            let fn_ptr_val = value_map
                .temp_values
                .get(id)
                .copied()
                .ok_or_else(|| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: format!("indirect callee temp {} not found", id.render()),
                })?;
            let fn_ptr = match fn_ptr_val {
                BasicValueEnum::PointerValue(pv) => pv,
                other => {
                    return Err(CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: format!("indirect callee temp {} is not a pointer: {:?}", id.render(), other.get_type()),
                    });
                }
            };
            // Build the LLVM function type from the arg types and return type.
            let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = args
                .iter()
                .map(|arg| match arg.ty() {
                    IrType::Ref(_) => Ok(self.cg.context.ptr_type(inkwell::AddressSpace::default()).into()),
                    ty => self.lower_basic_type(&ty).map(Into::into),
                })
                .collect::<Result<_, _>>()?;
            let fn_type = match self
                .cg
                .lowerer
                .lower_return_type(ret_ty, Some(&self.cg.planner.named_struct_types))?
            {
                inkwell::types::AnyTypeEnum::VoidType(v) => v.fn_type(&param_types, false),
                inkwell::types::AnyTypeEnum::IntType(i) => i.fn_type(&param_types, false),
                inkwell::types::AnyTypeEnum::FloatType(f) => f.fn_type(&param_types, false),
                inkwell::types::AnyTypeEnum::PointerType(p) => p.fn_type(&param_types, false),
                other => {
                    return Err(CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: format!("indirect call produces unsupported LLVM return type {:?}", other),
                    });
                }
            };
            let call = self
                .cg
                .builder
                .build_indirect_call(fn_type, fn_ptr, &call_args, &call_label)
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: e.to_string(),
                })?;
            if let Some(dst) = dst {
                if *ret_ty != IrType::Void {
                    let value = call.try_as_basic_value().unwrap_basic();
                    value_map.temp_values.insert(dst, value);
                }
            }
            return Ok(());
        }

        let function = match callee {
            IrValue::GlobalRef(name, _) => self
                .cg
                .planner
                .functions
                .get(name)
                .copied()
                .ok_or_else(|| CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: format!("unknown direct callee '{name}'"),
                })?,
            IrValue::ImportRef(module, name, _) => {
                self.get_or_declare_imported_function(module, name, args, ret_ty)?
            }
            other => {
                return Err(CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: format!("unsupported call target {other:?}"),
                });
            }
        };

        let call = self
            .cg
            .builder
            .build_call(function, &call_args, &call_label)
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_instr",
                detail: e.to_string(),
            })?;

        if let Some(dst) = dst {
            if *ret_ty != IrType::Void {
                let value = call.try_as_basic_value().unwrap_basic();
                value_map.temp_values.insert(dst, value);
            }
        }

        Ok(())
    }

    fn get_or_declare_imported_function(
        &self,
        module: &str,
        name: &str,
        args: &[IrValue],
        ret_ty: &IrType,
    ) -> Result<FunctionValue<'ctx>, CodegenError> {
        let symbol_name = CodegenOptions::public_symbol_name(module, name);
        if let Some(function) = self.cg.module.get_function(&symbol_name) {
            return Ok(function);
        }

        let param_types = args
            .iter()
            .map(|arg| self.lower_basic_type(&arg.ty()).map(Into::into))
            .collect::<Result<Vec<_>, _>>()?;

        let fn_type = match self
            .cg
            .lowerer
            .lower_return_type(ret_ty, Some(&self.cg.planner.named_struct_types))?
        {
            inkwell::types::AnyTypeEnum::VoidType(v) => v.fn_type(&param_types, false),
            inkwell::types::AnyTypeEnum::IntType(i) => i.fn_type(&param_types, false),
            inkwell::types::AnyTypeEnum::FloatType(f) => f.fn_type(&param_types, false),
            inkwell::types::AnyTypeEnum::PointerType(p) => p.fn_type(&param_types, false),
            other => {
                return Err(CodegenError::Unsupported {
                    stage: "emit_instr",
                    detail: format!(
                        "imported call '{symbol_name}' produces unsupported LLVM return type {:?}",
                        other
                    ),
                });
            }
        };

        Ok(self.cg.module.add_function(
            &symbol_name,
            fn_type,
            Some(inkwell::module::Linkage::External),
        ))
    }

    fn get_or_declare_imported_global_slot(
        &self,
        module: &str,
        name: &str,
    ) -> PointerValue<'ctx> {
        let symbol_name = CodegenOptions::public_symbol_name(module, name);
        if let Some(global) = self.cg.module.get_global(&symbol_name) {
            return global.as_pointer_value();
        }

        let ptr_ty = self.cg.context.ptr_type(AddressSpace::default());
        let global = self.cg.module.add_global(ptr_ty, None, &symbol_name);
        global.set_linkage(Linkage::External);
        global.as_pointer_value()
    }

    fn const_int(&self, value: i128, ty: &IrType) -> inkwell::values::IntValue<'ctx> {
        let (llvm_ty, sign_extend) = match ty {
            IrType::I8 => (self.cg.context.i8_type(), true),
            IrType::I16 => (self.cg.context.i16_type(), true),
            IrType::I32 => (self.cg.context.i32_type(), true),
            IrType::I64 => (self.cg.context.i64_type(), true),
            IrType::U8 | IrType::ShortChar => (self.cg.context.i8_type(), false),
            IrType::U16 => (self.cg.context.i16_type(), false),
            IrType::U32 | IrType::Char | IrType::Set(32) => (self.cg.context.i32_type(), false),
            IrType::Bool => (self.cg.context.bool_type(), false),
            _ => (self.cg.context.i64_type(), true),
        };
        llvm_ty.const_int(value as u64, sign_extend)
    }

    fn int_predicate(
        &self,
        value: &IrValue,
        signed: IntPredicate,
        unsigned: IntPredicate,
    ) -> IntPredicate {
        if self.is_unsigned_type(&value.ty()) {
            unsigned
        } else {
            signed
        }
    }

    fn is_unsigned_type(&self, ty: &IrType) -> bool {
        matches!(ty, IrType::U8 | IrType::U16 | IrType::U32 | IrType::U64 | IrType::Bool | IrType::Char | IrType::ShortChar | IrType::Set(_))
    }

    /// Emit an IS type test for `Instr::TypeCheck` and `Terminator::TypeTest`.
    ///
    /// - Named type → calls `@__newcp_type_test(obj_ptr, typedesc_ptr) -> i1`
    /// - Opaque fallback → emits constant `false` (unresolved type at lower time)
    /// Emit a `getelementptr inbounds` for a named-record field access.
    ///
    /// `base` must be a `ptr` to an instance of a named record type.
    /// The caller supplies `field_index` (0-based, already flattened with inherited fields)
    /// and `result_ty` (the value type of the field).
    ///
    /// The LLVM struct type is looked up from `planner.named_struct_types`.  If the
    /// struct type is not yet declared (e.g. for imported or unresolved named types),
    /// we fall back to a byte-offset-free opaque GEP that yields `ptr` (which will
    /// produce incorrect code but avoids a hard error during the first-slice bring-up).
    fn emit_gep(
        &mut self,
        dst: TempId,
        base: &IrValue,
        field_index: u32,
        _result_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        let base_ptr = self.resolve_pointer(base, value_map)?;
        let struct_ty = named_struct_type_from_ir_type(&base.ty(), &self.cg.planner.named_struct_types)
            .filter(|st| st.count_fields() > field_index);

        let zero = self.cg.context.i32_type().const_zero();
        let idx = self.cg.context.i32_type().const_int(field_index as u64, false);

        let field_ptr = if let Some(sty) = struct_ty {
            unsafe {
                self.cg
                    .builder
                    .build_in_bounds_gep(sty, base_ptr, &[zero, idx], &format!("gep.{field_index}"))
            }
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_gep",
                detail: e.to_string(),
            })?
        } else {
            // Fallback: we cannot emit a typed GEP without the struct type.
            // Emit an identity pointer and hope the generated code is corrected later.
            base_ptr
        };

        value_map.temp_values.insert(dst, field_ptr.into());
        Ok(())
    }

    fn emit_index_gep(
        &mut self,
        dst: TempId,
        base: &IrValue,
        index: &IrValue,
        element_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        let base_ptr = self.resolve_pointer(base, value_map)?;
        let elem_llvm_ty = self.lower_basic_type(element_ty)?;
        let idx_val = self.resolve_basic_value(index, value_map)?;
        let idx_int = if let Some(iv) = idx_val.into_int_value().try_into().ok() {
            iv
        } else {
            // Widen to i64 if not already an integer — safety fallback.
            idx_val.into_int_value()
        };
        let elem_ptr = unsafe {
            self.cg
                .builder
                .build_gep(elem_llvm_ty, base_ptr, &[idx_int], "arr_elem")
        }
        .map_err(|e| CodegenError::Unsupported {
            stage: "emit_index_gep",
            detail: e.to_string(),
        })?;
        value_map.temp_values.insert(dst, elem_ptr.into());
        Ok(())
    }

    fn emit_type_check(
        &mut self,
        value: &IrValue,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let IrType::Named(type_name) = ty else {
            // Opaque("is-check") is emitted by lower.rs when the target type was
            // unresolved.  Produce constant false until the ABI is defined.
            return Ok(self.cg.context.bool_type().const_zero().into());
        };

        // The subject can come in two shapes:
        //   - a value (Temp, loaded pointer): resolve_basic_value
        //   - a Ref-typed designator (e.g. GlobalRef("msg") for a
        //     VAR record param): resolve_pointer, which handles
        //     ref_param_slots indirection so the address we end up
        //     with IS the record's payload pointer.
        let obj_ptr: BasicValueEnum<'ctx> = if matches!(value.ty(), IrType::Ref(_)) {
            self.resolve_pointer(value, value_map)?.into()
        } else {
            self.resolve_basic_value(value, value_map)?
        };

        // Local types — `<Type>.desc` is emitted in this module, hand
        // back the global pointer.  Cross-module types — route through
        // the runtime TypeDesc registry the same way cross-module
        // `NEW` does, since the TypeDesc global lives in the other
        // module under a different mangled name (e.g. `SequencerDesc.desc`,
        // not `Sequencers.SequencerDesc.desc`).
        let typedesc_ptr = if type_name.contains('.')
            && !self.cg.planner.type_desc_globals.contains_key(type_name)
        {
            let lookup = self.get_or_declare_lookup_typedesc();
            let name_global = self.cg.get_or_emit_cstring_constant(type_name);
            let lookup_call = self
                .cg
                .builder
                .build_call(lookup, &[name_global.into()], "typedesc.is")
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_type_check",
                    detail: e.to_string(),
                })?;
            let typedesc_i64 = lookup_call
                .try_as_basic_value()
                .unwrap_basic()
                .into_int_value();
            self.cg
                .builder
                .build_int_to_ptr(
                    typedesc_i64,
                    self.cg.context.ptr_type(inkwell::AddressSpace::default()),
                    "typedesc.is.ptr",
                )
                .map_err(|e| CodegenError::Unsupported {
                    stage: "emit_type_check",
                    detail: e.to_string(),
                })?
        } else {
            self.get_or_declare_typedesc(type_name)
        };

        let type_test_fn = self.get_or_declare_type_test_fn();
        let call = self
            .cg
            .builder
            .build_call(
                type_test_fn,
                &[obj_ptr.into(), typedesc_ptr.into()],
                "typetest",
            )
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_type_check",
                detail: e.to_string(),
            })?;
        Ok(call.try_as_basic_value().unwrap_basic())
    }

    /// Get or declare `@__newcp_type_test(ptr, ptr) -> i1` — part of the runtime ABI.
    fn get_or_declare_type_test_fn(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_type_test";
        if let Some(f) = self.cg.module.get_function(NAME) {
            return f;
        }
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i1_ty = self.cg.context.bool_type();
        let fn_ty = i1_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.cg
            .module
            .add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// Get or declare `@__newcp_safepoint() -> void`.  Codegen emits
    /// a call to this at every CP procedure entry as a cooperative
    /// GC poll point.
    fn get_or_declare_safepoint(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_safepoint";
        if let Some(f) = self.cg.module.get_function(NAME) {
            return f;
        }
        let void_ty = self.cg.context.void_type();
        let fn_ty = void_ty.fn_type(&[], false);
        self.cg
            .module
            .add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// Get or declare `@__newcp_lookup_typedesc(ptr) -> i64` —
    /// used by cross-module `NEW(p)` to resolve a TypeDesc address
    /// via the runtime registry by qualified name.
    fn get_or_declare_lookup_typedesc(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_lookup_typedesc";
        if let Some(f) = self.cg.module.get_function(NAME) {
            return f;
        }
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let fn_ty = i64_ty.fn_type(&[ptr_ty.into()], false);
        self.cg
            .module
            .add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// Get or declare `@__newcp_string_eq_char(ptr, ptr) -> i64`.  The
    /// runtime walks two NUL-terminated UTF-32 buffers and returns 1
    /// when their codepoint sequences match, 0 otherwise.
    fn get_or_declare_string_eq_char(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_string_eq_char";
        if let Some(f) = self.cg.module.get_function(NAME) {
            return f;
        }
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let fn_ty = i64_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.cg
            .module
            .add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// SHORTCHAR (Latin-1) twin of `__newcp_string_eq_char`.
    fn get_or_declare_string_eq_shortchar(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_string_eq_shortchar";
        if let Some(f) = self.cg.module.get_function(NAME) {
            return f;
        }
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let fn_ty = i64_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.cg
            .module
            .add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// Emit `Instr::StringCompare` — walk two NUL-terminated CHAR /
    /// SHORTCHAR buffers via a runtime helper, then map the i64
    /// return into an i1 result per the requested comparison.
    ///
    /// `Eq` / `Ne` use the existing `__newcp_string_eq_*` helpers
    /// (1/0 return).  The four ordering ops route through
    /// `__newcp_string_cmp_*` (-1/0/1 return) and are chained with
    /// an integer compare against 0.
    fn emit_string_compare_instr(
        &mut self,
        dst: TempId,
        lhs: &IrValue,
        rhs: &IrValue,
        op: newcp_ir::StringCmpOp,
        elem_is_short: bool,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        use inkwell::IntPredicate;
        use newcp_ir::StringCmpOp;
        let lhs_ptr = self.resolve_string_operand_ptr(lhs, value_map)?;
        let rhs_ptr = self.resolve_string_operand_ptr(rhs, value_map)?;
        let is_eq_class = matches!(op, StringCmpOp::Eq | StringCmpOp::Ne);
        let helper = if is_eq_class {
            if elem_is_short {
                self.get_or_declare_string_eq_shortchar()
            } else {
                self.get_or_declare_string_eq_char()
            }
        } else if elem_is_short {
            self.get_or_declare_string_cmp_shortchar()
        } else {
            self.get_or_declare_string_cmp_char()
        };
        let label = match op {
            StringCmpOp::Eq => "streq",
            StringCmpOp::Ne => "strne",
            StringCmpOp::Lt => "strlt",
            StringCmpOp::Le => "strle",
            StringCmpOp::Gt => "strgt",
            StringCmpOp::Ge => "strge",
        };
        let call = self
            .cg
            .builder
            .build_call(helper, &[lhs_ptr.into(), rhs_ptr.into()], label)
            .map_err(|e| CodegenError::Unsupported {
                stage: "emit_string_compare",
                detail: e.to_string(),
            })?;
        let helper_i64 = call.try_as_basic_value().unwrap_basic().into_int_value();
        let result = match op {
            StringCmpOp::Eq | StringCmpOp::Ne => {
                let i1_eq = self
                    .cg
                    .builder
                    .build_int_truncate(helper_i64, self.cg.context.bool_type(), "streq.bool")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_string_compare",
                        detail: e.to_string(),
                    })?;
                if matches!(op, StringCmpOp::Eq) {
                    i1_eq
                } else {
                    self.cg
                        .builder
                        .build_not(i1_eq, "strne")
                        .map_err(|e| CodegenError::Unsupported {
                            stage: "emit_string_compare",
                            detail: e.to_string(),
                        })?
                }
            }
            ord_op => {
                let predicate = match ord_op {
                    StringCmpOp::Lt => IntPredicate::SLT,
                    StringCmpOp::Le => IntPredicate::SLE,
                    StringCmpOp::Gt => IntPredicate::SGT,
                    StringCmpOp::Ge => IntPredicate::SGE,
                    _ => unreachable!(),
                };
                let zero = self.cg.context.i64_type().const_int(0, true);
                self.cg
                    .builder
                    .build_int_compare(predicate, helper_i64, zero, "strord")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_string_compare",
                        detail: e.to_string(),
                    })?
            }
        };
        value_map.temp_values.insert(dst, result.into());
        Ok(())
    }

    /// Get or declare `@__newcp_string_cmp_char(ptr, ptr) -> i64`.
    /// Returns -1/0/1 for lexicographic ordering on NUL-terminated
    /// CHAR (UTF-32) buffers.
    fn get_or_declare_string_cmp_char(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_string_cmp_char";
        if let Some(function) = self.cg.module.get_function(NAME) {
            return function;
        }
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let fn_ty = i64_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.cg.module.add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// SHORTCHAR (Latin-1) twin of `get_or_declare_string_cmp_char`.
    fn get_or_declare_string_cmp_shortchar(&self) -> FunctionValue<'ctx> {
        const NAME: &str = "__newcp_string_cmp_shortchar";
        if let Some(function) = self.cg.module.get_function(NAME) {
            return function;
        }
        let ptr_ty = self.cg.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = self.cg.context.i64_type();
        let fn_ty = i64_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.cg.module.add_function(NAME, fn_ty, Some(inkwell::module::Linkage::External))
    }

    /// Resolve a string-compare operand to a pointer.  ConstStr →
    /// global string constant; designator-address (Ref-typed
    /// IrValue) → its alloca/GEP pointer.  Other shapes fall back
    /// to the basic-value resolver and we attempt to extract a
    /// pointer; if none is available, NULL is passed (the runtime
    /// helper handles NULL inputs as "unequal").
    fn resolve_string_operand_ptr(
        &mut self,
        value: &IrValue,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        if let IrValue::ConstStr(s, elem_ty) = value {
            return Ok(self.cg.get_or_emit_string_constant(s, elem_ty));
        }
        if matches!(value.ty(), IrType::Ref(_)) {
            return self.resolve_pointer(value, value_map);
        }
        let basic = self.resolve_basic_value(value, value_map)?;
        if basic.is_pointer_value() {
            Ok(basic.into_pointer_value())
        } else {
            Ok(self
                .cg
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null())
        }
    }

    /// Get or declare a TypeDesc global pointer for `type_name`.
    ///
    /// Resolution order:
    /// 1. **Local types** — look up `planner.type_desc_globals` for a
    ///    TypeDesc emitted in this module (named `<TypeName>.desc`).
    ///    The global is real; we just hand back the pointer.
    /// 2. **Cross-module types** — emit a `<Module>.<Type>.desc`
    ///    external declaration that the JIT linker resolves to the
    ///    other module's TypeDesc, falling back to a runtime lookup
    ///    via `__newcp_lookup_typedesc` at use time.  For now we use
    ///    the loader's external-symbol mechanism; see how cross-
    ///    module method calls handle the same problem.
    /// 3. **Unknown** — declare a placeholder global so codegen
    ///    proceeds; the JIT will fail to link rather than crash.
    fn get_or_declare_typedesc(&self, type_name: &str) -> PointerValue<'ctx> {
        // Local TypeDesc — we already emitted `<Type>.desc`.
        if let Some(g) = self.cg.planner.type_desc_globals.get(type_name) {
            return g.as_pointer_value();
        }
        // Already declared (e.g. on a previous call within this module).
        let desc_name = format!("{type_name}.desc");
        if let Some(g) = self.cg.module.get_global(&desc_name) {
            return g.as_pointer_value();
        }
        // Cross-module / unresolved: declare an external `<Type>.desc`.
        // The JIT side resolves these via the loader's per-module
        // export table, which exposes every emitted TypeDesc global.
        let i8_ty = self.cg.context.i8_type();
        let global = self.cg.module.add_global(i8_ty, None, &desc_name);
        global.set_linkage(inkwell::module::Linkage::External);
        global.as_pointer_value()
    }
}

fn named_struct_type_from_ir_type<'ctx>(
    ty: &IrType,
    named_struct_types: &HashMap<String, StructType<'ctx>>,
) -> Option<StructType<'ctx>> {
    match ty {
        IrType::Named(name) => named_struct_types.get(name).copied(),
        IrType::Ref(inner) | IrType::Ptr(inner) | IrType::UntaggedPtr(inner) => {
            named_struct_type_from_ir_type(inner, named_struct_types)
        }
        _ => None,
    }
}

