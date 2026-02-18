use std::collections::BTreeMap;
use std::sync::Arc;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, UserFuncName};
use cranelift_codegen::isa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::DataDescription;
use cranelift_module::{default_libcall_names, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;
use vibe_mir::{MirExpr, MirFunction, MirProgram, MirStmt, MirType};

#[derive(Debug, Clone)]
pub struct CodegenOptions {
    pub target: String,
    pub profile: String,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "dev".to_string(),
        }
    }
}

pub fn emit_object(program: &MirProgram, options: &CodegenOptions) -> Result<Vec<u8>, String> {
    let triple = parse_target(&options.target)?;
    let isa = build_isa(triple, &options.profile)?;
    let builder = ObjectBuilder::new(isa, "vibe_module".to_string(), default_libcall_names())
        .map_err(|e| format!("failed to create object builder: {e}"))?;
    let mut module = ObjectModule::new(builder);

    let ptr_ty = module.target_config().pointer_type();
    let print_fn = declare_print_runtime(&mut module, ptr_ty)?;

    let mut function_ids = BTreeMap::new();
    let mut function_returns = BTreeMap::new();
    for f in &program.functions {
        let sig = build_signature(&module, f, ptr_ty);
        let linkage = if f.name == "main" || f.is_public {
            Linkage::Export
        } else {
            Linkage::Local
        };
        let id = module
            .declare_function(&f.name, linkage, &sig)
            .map_err(|e| format!("failed to declare function `{}`: {e}", f.name))?;
        function_ids.insert(f.name.clone(), id);
        function_returns.insert(f.name.clone(), f.return_type.clone());
    }

    for f in &program.functions {
        define_function(
            &mut module,
            f,
            *function_ids
                .get(&f.name)
                .ok_or_else(|| format!("missing function id for `{}`", f.name))?,
            &function_ids,
            &function_returns,
            print_fn,
            ptr_ty,
        )?;
    }

    let product = module.finish();
    product
        .emit()
        .map_err(|e| format!("failed to emit object bytes: {e}"))
}

fn parse_target(raw: &str) -> Result<Triple, String> {
    if raw == "host" {
        return Ok(Triple::host());
    }
    raw.parse::<Triple>()
        .map_err(|e| format!("invalid target triple `{raw}`: {e}"))
}

fn build_isa(triple: Triple, profile: &str) -> Result<Arc<dyn isa::TargetIsa>, String> {
    let mut flags_builder = settings::builder();
    flags_builder
        .set(
            "opt_level",
            if profile == "release" {
                "speed"
            } else {
                "none"
            },
        )
        .map_err(|e| format!("failed to set opt level: {e}"))?;
    flags_builder
        .set("is_pic", "true")
        .map_err(|e| format!("failed to set PIC flag: {e}"))?;
    let flags = settings::Flags::new(flags_builder);
    isa::lookup(triple)
        .map_err(|e| format!("unsupported target ISA: {e}"))?
        .finish(flags)
        .map_err(|e| format!("failed to build target ISA: {e}"))
}

fn build_signature(module: &ObjectModule, f: &MirFunction, ptr_ty: ir::Type) -> ir::Signature {
    let mut sig = module.make_signature();
    for param in &f.params {
        sig.params
            .push(AbiParam::new(mir_ty_to_clif(&param.ty, ptr_ty)));
    }
    if f.return_type != MirType::Void {
        sig.returns
            .push(AbiParam::new(mir_ty_to_clif(&f.return_type, ptr_ty)));
    }
    sig
}

fn declare_print_runtime(module: &mut ObjectModule, ptr_ty: ir::Type) -> Result<FuncId, String> {
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(ptr_ty));
    module
        .declare_function("vibe_println", Linkage::Import, &sig)
        .map_err(|e| format!("failed to declare runtime print symbol: {e}"))
}

fn define_function(
    module: &mut ObjectModule,
    function: &MirFunction,
    func_id: FuncId,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    print_fn: FuncId,
    ptr_ty: ir::Type,
) -> Result<(), String> {
    let mut ctx = module.make_context();
    ctx.func.signature = build_signature(module, function, ptr_ty);
    ctx.func.name = UserFuncName::user(0, func_id.as_u32());

    let mut builder_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);
    builder.seal_block(entry);

    let mut locals: BTreeMap<String, Variable> = BTreeMap::new();
    for (idx, param) in function.params.iter().enumerate() {
        let var = Variable::from_u32(idx as u32);
        let ty = mir_ty_to_clif(&param.ty, ptr_ty);
        builder.declare_var(var, ty);
        let value = builder.block_params(entry)[idx];
        builder.def_var(var, value);
        locals.insert(param.name.clone(), var);
    }
    let mut next_var = function.params.len();
    let mut str_data_counter = 0usize;

    let mut terminated = false;
    for stmt in &function.body {
        if terminated {
            break;
        }
        terminated = emit_stmt(
            stmt,
            module,
            &mut builder,
            &mut locals,
            &mut next_var,
            function_ids,
            function_returns,
            print_fn,
            ptr_ty,
            &mut str_data_counter,
            function,
        )?;
    }

    if !terminated {
        if function.return_type == MirType::Void {
            builder.ins().return_(&[]);
        } else {
            let default_ret = default_value(&mut builder, &function.return_type, ptr_ty);
            builder.ins().return_(&[default_ret]);
        }
    }

    builder.finalize();
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| format!("failed to define function `{}`: {e}", function.name))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_stmt(
    stmt: &MirStmt,
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &mut BTreeMap<String, Variable>,
    next_var: &mut usize,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    print_fn: FuncId,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<bool, String> {
    match stmt {
        MirStmt::Let { name, expr } => {
            let value = emit_expr(
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            let var = Variable::from_u32(*next_var as u32);
            *next_var += 1;
            builder.declare_var(var, value_type_for_expr(expr, ptr_ty));
            builder.def_var(var, value);
            locals.insert(name.clone(), var);
            Ok(false)
        }
        MirStmt::Assign { name, expr } => {
            let value = emit_expr(
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            let var = if let Some(v) = locals.get(name) {
                *v
            } else {
                let v = Variable::from_u32(*next_var as u32);
                *next_var += 1;
                builder.declare_var(v, value_type_for_expr(expr, ptr_ty));
                locals.insert(name.clone(), v);
                v
            };
            builder.def_var(var, value);
            Ok(false)
        }
        MirStmt::Expr(expr) => {
            let _ = emit_expr(
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            Ok(false)
        }
        MirStmt::Return(expr) => {
            let value = emit_expr(
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            if owner.return_type == MirType::Void {
                builder.ins().return_(&[]);
            } else {
                builder.ins().return_(&[value]);
            }
            Ok(true)
        }
        MirStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let cond_v = emit_expr(
                cond,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            let cond_ty = builder.func.dfg.value_type(cond_v);
            let zero = builder.ins().iconst(cond_ty, 0);
            let cond_b = builder.ins().icmp(IntCC::NotEqual, cond_v, zero);

            let then_block = builder.create_block();
            let else_block = builder.create_block();
            let merge_block = builder.create_block();
            builder.ins().brif(cond_b, then_block, &[], else_block, &[]);

            builder.switch_to_block(then_block);
            let mut then_terminated = false;
            for s in then_body {
                if then_terminated {
                    break;
                }
                then_terminated = emit_stmt(
                    s,
                    module,
                    builder,
                    locals,
                    next_var,
                    function_ids,
                    function_returns,
                    print_fn,
                    ptr_ty,
                    str_data_counter,
                    owner,
                )?;
            }
            if !then_terminated {
                builder.ins().jump(merge_block, &[]);
            }
            builder.seal_block(then_block);

            builder.switch_to_block(else_block);
            let mut else_terminated = false;
            for s in else_body {
                if else_terminated {
                    break;
                }
                else_terminated = emit_stmt(
                    s,
                    module,
                    builder,
                    locals,
                    next_var,
                    function_ids,
                    function_returns,
                    print_fn,
                    ptr_ty,
                    str_data_counter,
                    owner,
                )?;
            }
            if !else_terminated {
                builder.ins().jump(merge_block, &[]);
            }
            builder.seal_block(else_block);

            if then_terminated && else_terminated {
                Ok(true)
            } else {
                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                Ok(false)
            }
        }
        MirStmt::While { .. }
        | MirStmt::Repeat { .. }
        | MirStmt::Select { .. }
        | MirStmt::Go(_) => {
            Err("codegen for this control-flow construct is not yet implemented".to_string())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_expr(
    expr: &MirExpr,
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &BTreeMap<String, Variable>,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    print_fn: FuncId,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<ir::Value, String> {
    Ok(match expr {
        MirExpr::Var(name) => {
            let var = locals
                .get(name)
                .ok_or_else(|| format!("unknown local `{name}` in function `{}`", owner.name))?;
            builder.use_var(*var)
        }
        MirExpr::Int(v) => builder.ins().iconst(ir::types::I64, *v),
        MirExpr::Float(v) => builder.ins().f64const(*v),
        MirExpr::Bool(v) => builder.ins().iconst(ir::types::I8, i64::from(*v)),
        MirExpr::Str(s) => emit_string_data(module, builder, s, ptr_ty, str_data_counter, owner)?,
        MirExpr::List(_) | MirExpr::Map(_) => {
            return Err("list/map codegen not implemented yet".to_string())
        }
        MirExpr::Member { object, field } => {
            let _ = emit_expr(
                object,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            return Err(format!(
                "member access `{field}` codegen not implemented yet"
            ));
        }
        MirExpr::Call { callee, args } => {
            if let MirExpr::Var(name) = &**callee {
                if name == "print" || name == "println" {
                    if args.len() != 1 {
                        return Err(format!("`{name}` expects one argument"));
                    }
                    let arg0 = emit_expr(
                        &args[0],
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        print_fn,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    let local_print = module.declare_func_in_func(print_fn, builder.func);
                    builder.ins().call(local_print, &[arg0]);
                    return Ok(builder.ins().iconst(ir::types::I64, 0));
                }
                if let Some(fid) = function_ids.get(name) {
                    let local = module.declare_func_in_func(*fid, builder.func);
                    let mut lowered_args = Vec::with_capacity(args.len());
                    for arg in args {
                        lowered_args.push(emit_expr(
                            arg,
                            module,
                            builder,
                            locals,
                            function_ids,
                            function_returns,
                            print_fn,
                            ptr_ty,
                            str_data_counter,
                            owner,
                        )?);
                    }
                    let call = builder.ins().call(local, &lowered_args);
                    if function_returns.get(name) == Some(&MirType::Void) {
                        return Ok(builder.ins().iconst(ir::types::I64, 0));
                    }
                    let results = builder.inst_results(call);
                    if let Some(v) = results.first() {
                        return Ok(*v);
                    }
                    return Ok(builder.ins().iconst(ir::types::I64, 0));
                }
                return Err(format!("unknown call target `{name}`"));
            }
            return Err("dynamic call targets are not supported in phase 2".to_string());
        }
        MirExpr::Binary { left, op, right } => {
            let l = emit_expr(
                left,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            let r = emit_expr(
                right,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            match op.as_str() {
                "Add" => builder.ins().iadd(l, r),
                "Sub" => builder.ins().isub(l, r),
                "Mul" => builder.ins().imul(l, r),
                "Div" => builder.ins().sdiv(l, r),
                "Eq" | "Ne" | "Lt" | "Le" | "Gt" | "Ge" => {
                    let cc = match op.as_str() {
                        "Eq" => IntCC::Equal,
                        "Ne" => IntCC::NotEqual,
                        "Lt" => IntCC::SignedLessThan,
                        "Le" => IntCC::SignedLessThanOrEqual,
                        "Gt" => IntCC::SignedGreaterThan,
                        "Ge" => IntCC::SignedGreaterThanOrEqual,
                        _ => IntCC::Equal,
                    };
                    let cmp = builder.ins().icmp(cc, l, r);
                    builder.ins().uextend(ir::types::I64, cmp)
                }
                _ => return Err(format!("unsupported binary op `{op}`")),
            }
        }
        MirExpr::Unary { op, expr } => {
            let v = emit_expr(
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                print_fn,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            match op.as_str() {
                "Neg" => builder.ins().ineg(v),
                "Not" => {
                    let v_ty = builder.func.dfg.value_type(v);
                    let zero = builder.ins().iconst(v_ty, 0);
                    let cmp = builder.ins().icmp(IntCC::Equal, v, zero);
                    builder.ins().uextend(ir::types::I64, cmp)
                }
                _ => return Err(format!("unsupported unary op `{op}`")),
            }
        }
        MirExpr::Question { expr } | MirExpr::Old { expr } => emit_expr(
            expr,
            module,
            builder,
            locals,
            function_ids,
            function_returns,
            print_fn,
            ptr_ty,
            str_data_counter,
            owner,
        )?,
        MirExpr::DotResult => builder.ins().iconst(ir::types::I64, 0),
    })
}

fn emit_string_data(
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    value: &str,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<ir::Value, String> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    let sym = format!("__vibe_str_{}_{}", owner.name, *str_data_counter);
    *str_data_counter += 1;

    let data_id = module
        .declare_data(&sym, Linkage::Local, false, false)
        .map_err(|e| format!("failed to declare string data `{sym}`: {e}"))?;
    let mut data_desc = DataDescription::new();
    data_desc.define(bytes.into_boxed_slice());
    module
        .define_data(data_id, &data_desc)
        .map_err(|e| format!("failed to define string data `{sym}`: {e}"))?;
    let local = module.declare_data_in_func(data_id, builder.func);
    Ok(builder.ins().symbol_value(ptr_ty, local))
}

fn default_value(builder: &mut FunctionBuilder<'_>, ty: &MirType, ptr_ty: ir::Type) -> ir::Value {
    match ty {
        MirType::I64 | MirType::Unknown => builder.ins().iconst(ir::types::I64, 0),
        MirType::F64 => builder.ins().f64const(0.0),
        MirType::Bool => builder.ins().iconst(ir::types::I8, 0),
        MirType::Str => builder.ins().iconst(ptr_ty, 0),
        MirType::Void => builder.ins().iconst(ir::types::I64, 0),
    }
}

fn value_type_for_expr(expr: &MirExpr, ptr_ty: ir::Type) -> ir::Type {
    match expr {
        MirExpr::Float(_) => ir::types::F64,
        MirExpr::Bool(_) => ir::types::I8,
        MirExpr::Str(_) => ptr_ty,
        _ => ir::types::I64,
    }
}

fn mir_ty_to_clif(ty: &MirType, ptr_ty: ir::Type) -> ir::Type {
    match ty {
        MirType::I64 | MirType::Unknown => ir::types::I64,
        MirType::F64 => ir::types::F64,
        MirType::Bool => ir::types::I8,
        MirType::Str => ptr_ty,
        MirType::Void => ir::types::I64,
    }
}

#[cfg(test)]
mod tests {
    use super::{emit_object, CodegenOptions};
    use vibe_mir::{MirExpr, MirFunction, MirProgram, MirStmt, MirType};

    #[test]
    fn emits_object_for_simple_program() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: MirType::I64,
                body: vec![
                    MirStmt::Expr(MirExpr::Call {
                        callee: Box::new(MirExpr::Var("println".to_string())),
                        args: vec![MirExpr::Str("hello".to_string())],
                    }),
                    MirStmt::Return(MirExpr::Int(0)),
                ],
            }],
        };
        let object = emit_object(&program, &CodegenOptions::default()).expect("object should emit");
        assert!(!object.is_empty(), "object bytes should not be empty");
    }

    #[test]
    fn object_emission_is_deterministic_for_same_program() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: MirType::I64,
                body: vec![MirStmt::Return(MirExpr::Int(0))],
            }],
        };
        let first = emit_object(&program, &CodegenOptions::default()).expect("first object");
        let second = emit_object(&program, &CodegenOptions::default()).expect("second object");
        assert_eq!(
            first, second,
            "object output should be stable for same input"
        );
    }
}
