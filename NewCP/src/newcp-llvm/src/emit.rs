use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

use newcp_ir::{BinOp, BlockId, IrProcedure, IrType, IrValue, TempId};

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
}

impl<'ctx> ValueMap<'ctx> {
    fn new() -> Self {
        Self {
            temp_values: HashMap::new(),
            block_map: HashMap::new(),
            named_slots: HashMap::new(),
            ref_param_slots: HashMap::new(),
        }
    }
}

/// Emits one procedure body into the `CodegenModule`.
pub struct ProcedureEmitter<'ctx, 'm> {
    cg: &'m mut CodegenModule<'ctx>,
    options: &'m CodegenOptions,
}

impl<'ctx, 'm> ProcedureEmitter<'ctx, 'm> {
    pub fn new(cg: &'m mut CodegenModule<'ctx>, options: &'m CodegenOptions) -> Self {
        Self { cg, options }
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

        let result = match instr {
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
            other => Err(CodegenError::Unsupported {
                stage: "emit_instr",
                detail: format!("{other:?}"),
            }),
        };

        match result {
            Ok(()) => Ok(()),
            Err(err) if self.options.strict_unsupported => Err(err),
            Err(_) => Ok(()),
        }
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
                // Emit a call to `llvm.trap` for any trap kind.
                let trap_fn = self.get_or_declare_llvm_trap();
                self.cg
                    .builder
                    .build_call(trap_fn, &[], "trap")
                    .map_err(|e| CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail: e.to_string(),
                    })?;
                // After llvm.trap, we still need a terminator. Emit unreachable.
                self.cg.builder.build_unreachable().map_err(|e| CodegenError::Unsupported {
                    stage: "emit_terminator",
                    detail: e.to_string(),
                })?;
                let _ = kind;
            }
            other => {
                let detail = format!("{other:?}");
                if self.options.strict_unsupported {
                    return Err(CodegenError::Unsupported {
                        stage: "emit_terminator",
                        detail,
                    });
                }
                // Non-strict fallback: emit unreachable so the block has a terminator.
                self.cg.builder.build_unreachable().map_err(|e| CodegenError::Unsupported {
                    stage: "emit_terminator",
                    detail: e.to_string(),
                })?;
            }
        }
        Ok(())
    }

    fn get_or_declare_llvm_trap(&self) -> inkwell::values::FunctionValue<'ctx> {
        if let Some(f) = self.cg.module.get_function("llvm.trap") {
            return f;
        }
        let void_ty = self.cg.context.void_type();
        let fn_ty = void_ty.fn_type(&[], false);
        self.cg.module.add_function("llvm.trap", fn_ty, None)
    }

    fn bind_proc_slots(
        &mut self,
        fn_val: FunctionValue<'ctx>,
        proc: &IrProcedure,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
        for (index, (name, ty)) in proc.params.iter().enumerate() {
            let slot = self.ensure_named_slot(name, ty, value_map)?;
            let param = fn_val.get_nth_param(index as u32).ok_or_else(|| CodegenError::Unsupported {
                stage: "emit",
                detail: format!("missing LLVM parameter {index} for '{}'", proc.name),
            })?;
            if matches!(ty, IrType::Ref(_)) {
                value_map.ref_param_slots.insert(name.clone(), slot);
            }
            self.cg
                .builder
                .position_at_end(value_map.block_map[&proc.entry]);
            self.cg.builder.build_store(slot, param).map_err(|e| CodegenError::Unsupported {
                stage: "emit",
                detail: e.to_string(),
            })?;
        }

        if proc.ret_ty != IrType::Void {
            let _ = self.ensure_named_slot("$result", &proc.ret_ty, value_map)?;
        }

        Ok(())
    }

    fn ensure_named_slot(
        &self,
        name: &str,
        ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        if let Some(slot) = value_map.named_slots.get(name).copied() {
            return Ok(slot);
        }

        let llvm_ty = self.lower_basic_type(ty)?;
        let entry_block = *value_map.block_map.values().min_by_key(|b| b.get_name().to_str().ok()).ok_or_else(|| {
            CodegenError::Unsupported {
                stage: "emit",
                detail: "no entry block available for slot allocation".to_string(),
            }
        })?;
        let slot_builder = self.cg.context.create_builder();
        if let Some(first_instr) = entry_block.get_first_instruction() {
            slot_builder.position_before(&first_instr);
        } else {
            slot_builder.position_at_end(entry_block);
        }
        let slot = slot_builder.build_alloca(llvm_ty, name).map_err(|e| CodegenError::Unsupported {
            stage: "emit",
            detail: e.to_string(),
        })?;
        value_map.named_slots.insert(name.to_string(), slot);
        Ok(slot)
    }

    fn lower_basic_type(&self, ty: &IrType) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        self.cg.lowerer.lower_basic(ty)
    }

    fn resolve_pointer(
        &self,
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
            IrValue::ImportRef(module, name, IrType::Ref(inner)) => {
                self.ensure_named_slot(&format!("{module}.{name}"), inner, value_map)
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
        &self,
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
            IrValue::ConstChar(v) => Ok(self.cg.context.i16_type().const_int(*v as u64, false).into()),
            IrValue::Null(_) => Ok(self.cg.context.ptr_type(inkwell::AddressSpace::default()).const_null().into()),
            other => Err(CodegenError::Unsupported {
                stage: "emit",
                detail: format!("unsupported value operand {other:?}"),
            }),
        }
    }

    fn emit_binop(
        &self,
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

        let lhs = left_value.into_int_value();
        let rhs = right_value.into_int_value();
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
                other => {
                    return Err(CodegenError::Unsupported {
                        stage: "emit_instr",
                        detail: format!("unsupported integer binop {other:?}"),
                    });
                }
            }
        }
        .map_err(|e| CodegenError::Unsupported {
            stage: "emit_instr",
            detail: e.to_string(),
        })?;

        let _ = ty;
        Ok(value)
    }

    fn emit_call(
        &self,
        dst: Option<TempId>,
        callee: &IrValue,
        args: &[IrValue],
        ret_ty: &IrType,
        value_map: &mut ValueMap<'ctx>,
    ) -> Result<(), CodegenError> {
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

        let call_args: Vec<BasicMetadataValueEnum<'ctx>> = args
            .iter()
            .map(|arg| match arg.ty() {
                IrType::Ref(_) => self.resolve_pointer(arg, value_map).map(Into::into),
                _ => self.resolve_basic_value(arg, value_map).map(Into::into),
            })
            .collect::<Result<_, _>>()?;
        let call = self
            .cg
            .builder
            .build_call(function, &call_args, dst.map(|id| id.render()).as_deref().unwrap_or("call"))
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
        let symbol_name = format!("{module}.{name}");
        if let Some(function) = self.cg.module.get_function(&symbol_name) {
            return Ok(function);
        }

        let param_types = args
            .iter()
            .map(|arg| self.lower_basic_type(&arg.ty()).map(Into::into))
            .collect::<Result<Vec<_>, _>>()?;

        let fn_type = match self.cg.lowerer.lower_return_type(ret_ty)? {
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

    fn const_int(&self, value: i128, ty: &IrType) -> inkwell::values::IntValue<'ctx> {
        let (llvm_ty, sign_extend) = match ty {
            IrType::I8 => (self.cg.context.i8_type(), true),
            IrType::I16 => (self.cg.context.i16_type(), true),
            IrType::I32 => (self.cg.context.i32_type(), true),
            IrType::I64 => (self.cg.context.i64_type(), true),
            IrType::U8 | IrType::ShortChar => (self.cg.context.i8_type(), false),
            IrType::U16 | IrType::Char => (self.cg.context.i16_type(), false),
            IrType::U32 | IrType::Set(32) => (self.cg.context.i32_type(), false),
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
        matches!(ty, IrType::U8 | IrType::U16 | IrType::U32 | IrType::Bool | IrType::Char | IrType::ShortChar | IrType::Set(_))
    }
}
