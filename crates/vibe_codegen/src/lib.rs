use std::collections::BTreeMap;
use std::sync::Arc;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, UserFuncName};
use cranelift_codegen::isa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Switch, Variable};
use cranelift_module::DataDescription;
use cranelift_module::{default_libcall_names, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;
use vibe_mir::{
    MirContractKind, MirExpr, MirFunction, MirProgram, MirSelectCase, MirSelectPattern, MirStmt,
    MirType,
};

#[derive(Debug, Clone)]
pub struct CodegenOptions {
    pub target: String,
    pub profile: String,
    pub debuginfo: String,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "dev".to_string(),
            debuginfo: "line".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
struct RuntimeFunctions {
    print_fn: FuncId,
    panic_fn: FuncId,
    list_new_i64_fn: FuncId,
    list_append_i64_fn: FuncId,
    container_len_fn: FuncId,
    container_get_i64_fn: FuncId,
    container_set_i64_fn: FuncId,
    container_contains_i64_fn: FuncId,
    container_remove_i64_fn: FuncId,
    map_new_i64_i64_fn: FuncId,
    map_new_str_i64_fn: FuncId,
    container_get_str_i64_fn: FuncId,
    container_set_str_i64_fn: FuncId,
    container_contains_str_i64_fn: FuncId,
    container_remove_str_i64_fn: FuncId,
    str_concat_fn: FuncId,
    chan_new_fn: FuncId,
    chan_send_fn: FuncId,
    chan_recv_fn: FuncId,
    chan_has_data_fn: FuncId,
    chan_close_fn: FuncId,
    chan_is_closed_fn: FuncId,
    spawn0_fn: FuncId,
    spawn1_i64_fn: FuncId,
    select_cursor_fn: FuncId,
    sleep_ms_fn: FuncId,
}

pub fn emit_object(program: &MirProgram, options: &CodegenOptions) -> Result<Vec<u8>, String> {
    let triple = parse_target(&options.target)?;
    let isa = build_isa(triple, &options.profile, &options.debuginfo)?;
    let builder = ObjectBuilder::new(isa, "vibe_module".to_string(), default_libcall_names())
        .map_err(|e| format!("failed to create object builder: {e}"))?;
    let mut module = ObjectModule::new(builder);

    let ptr_ty = module.target_config().pointer_type();
    let runtime_fns = declare_runtime_functions(&mut module, ptr_ty)?;

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
            runtime_fns,
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

fn build_isa(
    triple: Triple,
    profile: &str,
    debuginfo: &str,
) -> Result<Arc<dyn isa::TargetIsa>, String> {
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
    if debuginfo != "none" {
        let _ = flags_builder.set("unwind_info", "true");
    }
    if debuginfo == "full" {
        let _ = flags_builder.set("preserve_frame_pointers", "true");
    }
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

fn declare_runtime_functions(
    module: &mut ObjectModule,
    ptr_ty: ir::Type,
) -> Result<RuntimeFunctions, String> {
    let mut sig = module.make_signature();
    sig.params.push(AbiParam::new(ptr_ty));
    let print_fn = module
        .declare_function("vibe_println", Linkage::Import, &sig)
        .map_err(|e| format!("failed to declare runtime print symbol: {e}"))?;

    let panic_fn = module
        .declare_function("vibe_panic", Linkage::Import, &sig)
        .map_err(|e| format!("failed to declare runtime panic symbol: {e}"))?;

    let mut list_new_sig = module.make_signature();
    list_new_sig.params.push(AbiParam::new(ir::types::I64));
    list_new_sig.returns.push(AbiParam::new(ptr_ty));
    let list_new_i64_fn = module
        .declare_function("vibe_list_new_i64", Linkage::Import, &list_new_sig)
        .map_err(|e| format!("failed to declare runtime list_new symbol: {e}"))?;

    let mut list_append_sig = module.make_signature();
    list_append_sig.params.push(AbiParam::new(ptr_ty));
    list_append_sig.params.push(AbiParam::new(ir::types::I64));
    list_append_sig.returns.push(AbiParam::new(ir::types::I64));
    let list_append_i64_fn = module
        .declare_function("vibe_list_append_i64", Linkage::Import, &list_append_sig)
        .map_err(|e| format!("failed to declare runtime list_append symbol: {e}"))?;

    let mut map_new_i64_i64_sig = module.make_signature();
    map_new_i64_i64_sig.returns.push(AbiParam::new(ptr_ty));
    let map_new_i64_i64_fn = module
        .declare_function(
            "vibe_map_new_i64_i64",
            Linkage::Import,
            &map_new_i64_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime map_new_i64_i64 symbol: {e}"))?;

    let mut map_new_str_i64_sig = module.make_signature();
    map_new_str_i64_sig.returns.push(AbiParam::new(ptr_ty));
    let map_new_str_i64_fn = module
        .declare_function(
            "vibe_map_new_str_i64",
            Linkage::Import,
            &map_new_str_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime map_new_str_i64 symbol: {e}"))?;

    let mut str_concat_sig = module.make_signature();
    str_concat_sig.params.push(AbiParam::new(ptr_ty));
    str_concat_sig.params.push(AbiParam::new(ptr_ty));
    str_concat_sig.returns.push(AbiParam::new(ptr_ty));
    let str_concat_fn = module
        .declare_function("vibe_str_concat", Linkage::Import, &str_concat_sig)
        .map_err(|e| format!("failed to declare runtime str_concat symbol: {e}"))?;

    let mut container_len_sig = module.make_signature();
    container_len_sig.params.push(AbiParam::new(ptr_ty));
    container_len_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_len_fn = module
        .declare_function("vibe_container_len", Linkage::Import, &container_len_sig)
        .map_err(|e| format!("failed to declare runtime container_len symbol: {e}"))?;

    let mut container_get_i64_sig = module.make_signature();
    container_get_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_get_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_get_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_get_i64_fn = module
        .declare_function(
            "vibe_container_get_i64",
            Linkage::Import,
            &container_get_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_get_i64 symbol: {e}"))?;

    let mut container_set_i64_sig = module.make_signature();
    container_set_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_set_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_set_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_set_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_set_i64_fn = module
        .declare_function(
            "vibe_container_set_i64",
            Linkage::Import,
            &container_set_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_set_i64 symbol: {e}"))?;

    let mut container_contains_i64_sig = module.make_signature();
    container_contains_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_contains_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_contains_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_contains_i64_fn = module
        .declare_function(
            "vibe_container_contains_i64",
            Linkage::Import,
            &container_contains_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_contains_i64 symbol: {e}"))?;

    let mut container_remove_i64_sig = module.make_signature();
    container_remove_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_remove_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_remove_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_remove_i64_fn = module
        .declare_function(
            "vibe_container_remove_i64",
            Linkage::Import,
            &container_remove_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_remove_i64 symbol: {e}"))?;

    let mut container_get_str_i64_sig = module.make_signature();
    container_get_str_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_get_str_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_get_str_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_get_str_i64_fn = module
        .declare_function(
            "vibe_container_get_str_i64",
            Linkage::Import,
            &container_get_str_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_get_str_i64 symbol: {e}"))?;

    let mut container_set_str_i64_sig = module.make_signature();
    container_set_str_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_set_str_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_set_str_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_set_str_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_set_str_i64_fn = module
        .declare_function(
            "vibe_container_set_str_i64",
            Linkage::Import,
            &container_set_str_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_set_str_i64 symbol: {e}"))?;

    let mut container_contains_str_i64_sig = module.make_signature();
    container_contains_str_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_contains_str_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_contains_str_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_contains_str_i64_fn = module
        .declare_function(
            "vibe_container_contains_str_i64",
            Linkage::Import,
            &container_contains_str_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_contains_str_i64 symbol: {e}"))?;

    let mut container_remove_str_i64_sig = module.make_signature();
    container_remove_str_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_remove_str_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_remove_str_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_remove_str_i64_fn = module
        .declare_function(
            "vibe_container_remove_str_i64",
            Linkage::Import,
            &container_remove_str_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_remove_str_i64 symbol: {e}"))?;

    let mut chan_new_sig = module.make_signature();
    chan_new_sig.params.push(AbiParam::new(ir::types::I64));
    chan_new_sig.returns.push(AbiParam::new(ptr_ty));
    let chan_new_fn = module
        .declare_function("vibe_chan_new_i64", Linkage::Import, &chan_new_sig)
        .map_err(|e| format!("failed to declare runtime chan_new symbol: {e}"))?;

    let mut chan_send_sig = module.make_signature();
    chan_send_sig.params.push(AbiParam::new(ptr_ty));
    chan_send_sig.params.push(AbiParam::new(ir::types::I64));
    chan_send_sig.returns.push(AbiParam::new(ir::types::I64));
    let chan_send_fn = module
        .declare_function("vibe_chan_send_i64", Linkage::Import, &chan_send_sig)
        .map_err(|e| format!("failed to declare runtime chan_send symbol: {e}"))?;

    let mut chan_recv_sig = module.make_signature();
    chan_recv_sig.params.push(AbiParam::new(ptr_ty));
    chan_recv_sig.returns.push(AbiParam::new(ir::types::I64));
    let chan_recv_fn = module
        .declare_function("vibe_chan_recv_i64", Linkage::Import, &chan_recv_sig)
        .map_err(|e| format!("failed to declare runtime chan_recv symbol: {e}"))?;

    let mut chan_has_data_sig = module.make_signature();
    chan_has_data_sig.params.push(AbiParam::new(ptr_ty));
    chan_has_data_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let chan_has_data_fn = module
        .declare_function(
            "vibe_chan_has_data_i64",
            Linkage::Import,
            &chan_has_data_sig,
        )
        .map_err(|e| format!("failed to declare runtime chan_has_data symbol: {e}"))?;

    let mut chan_close_sig = module.make_signature();
    chan_close_sig.params.push(AbiParam::new(ptr_ty));
    let chan_close_fn = module
        .declare_function("vibe_chan_close_i64", Linkage::Import, &chan_close_sig)
        .map_err(|e| format!("failed to declare runtime chan_close symbol: {e}"))?;

    let mut chan_closed_sig = module.make_signature();
    chan_closed_sig.params.push(AbiParam::new(ptr_ty));
    chan_closed_sig.returns.push(AbiParam::new(ir::types::I64));
    let chan_is_closed_fn = module
        .declare_function("vibe_chan_is_closed_i64", Linkage::Import, &chan_closed_sig)
        .map_err(|e| format!("failed to declare runtime chan_is_closed symbol: {e}"))?;

    let mut spawn0_sig = module.make_signature();
    spawn0_sig.params.push(AbiParam::new(ptr_ty));
    spawn0_sig.returns.push(AbiParam::new(ir::types::I64));
    let spawn0_fn = module
        .declare_function("vibe_spawn0", Linkage::Import, &spawn0_sig)
        .map_err(|e| format!("failed to declare runtime spawn0 symbol: {e}"))?;

    let mut spawn1_i64_sig = module.make_signature();
    spawn1_i64_sig.params.push(AbiParam::new(ptr_ty));
    spawn1_i64_sig.params.push(AbiParam::new(ir::types::I64));
    spawn1_i64_sig.returns.push(AbiParam::new(ir::types::I64));
    let spawn1_i64_fn = module
        .declare_function("vibe_spawn1_i64", Linkage::Import, &spawn1_i64_sig)
        .map_err(|e| format!("failed to declare runtime spawn1_i64 symbol: {e}"))?;

    let mut select_cursor_sig = module.make_signature();
    select_cursor_sig.params.push(AbiParam::new(ir::types::I64));
    select_cursor_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let select_cursor_fn = module
        .declare_function(
            "vibe_select_next_cursor",
            Linkage::Import,
            &select_cursor_sig,
        )
        .map_err(|e| format!("failed to declare runtime select_cursor symbol: {e}"))?;

    let mut sleep_sig = module.make_signature();
    sleep_sig.params.push(AbiParam::new(ir::types::I64));
    let sleep_ms_fn = module
        .declare_function("vibe_sleep_ms", Linkage::Import, &sleep_sig)
        .map_err(|e| format!("failed to declare runtime sleep symbol: {e}"))?;

    Ok(RuntimeFunctions {
        print_fn,
        panic_fn,
        list_new_i64_fn,
        list_append_i64_fn,
        container_len_fn,
        container_get_i64_fn,
        container_set_i64_fn,
        container_contains_i64_fn,
        container_remove_i64_fn,
        map_new_i64_i64_fn,
        map_new_str_i64_fn,
        container_get_str_i64_fn,
        container_set_str_i64_fn,
        container_contains_str_i64_fn,
        container_remove_str_i64_fn,
        str_concat_fn,
        chan_new_fn,
        chan_send_fn,
        chan_recv_fn,
        chan_has_data_fn,
        chan_close_fn,
        chan_is_closed_fn,
        spawn0_fn,
        spawn1_i64_fn,
        select_cursor_fn,
        sleep_ms_fn,
    })
}

fn define_function(
    module: &mut ObjectModule,
    function: &MirFunction,
    func_id: FuncId,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
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
            runtime_fns,
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
    runtime_fns: RuntimeFunctions,
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
                runtime_fns,
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
                runtime_fns,
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
                runtime_fns,
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
                runtime_fns,
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
        MirStmt::ContractCheck { kind, expr } => {
            emit_contract_check(
                kind,
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            Ok(false)
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
                runtime_fns,
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
                    runtime_fns,
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
                    runtime_fns,
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
        MirStmt::Go(expr) => {
            emit_go_stmt(
                expr,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            Ok(false)
        }
        MirStmt::Select { cases } => {
            emit_select_stmt(
                cases,
                module,
                builder,
                locals,
                next_var,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            Ok(false)
        }
        MirStmt::While { cond, body } => {
            emit_while_stmt(
                cond,
                body,
                module,
                builder,
                locals,
                next_var,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            Ok(false)
        }
        MirStmt::Repeat { count, body } => {
            emit_repeat_stmt(
                count,
                body,
                module,
                builder,
                locals,
                next_var,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            Ok(false)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_contract_check(
    kind: &MirContractKind,
    expr: &MirExpr,
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &BTreeMap<String, Variable>,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<(), String> {
    let cond_v = emit_expr(
        expr,
        module,
        builder,
        locals,
        function_ids,
        function_returns,
        runtime_fns,
        ptr_ty,
        str_data_counter,
        owner,
    )?;
    let cond_ty = builder.func.dfg.value_type(cond_v);
    let zero = builder.ins().iconst(cond_ty, 0);
    let cond_b = builder.ins().icmp(IntCC::NotEqual, cond_v, zero);
    let pass_block = builder.create_block();
    let fail_block = builder.create_block();
    builder.ins().brif(cond_b, pass_block, &[], fail_block, &[]);

    builder.switch_to_block(fail_block);
    let message = match kind {
        MirContractKind::Require => "contract @require failed in native execution",
        MirContractKind::Ensure => "contract @ensure failed in native execution",
    };
    let message_ptr = emit_string_data(module, builder, message, ptr_ty, str_data_counter, owner)?;
    let panic_local = module.declare_func_in_func(runtime_fns.panic_fn, builder.func);
    builder.ins().call(panic_local, &[message_ptr]);
    if owner.return_type == MirType::Void {
        builder.ins().return_(&[]);
    } else {
        let fallback = default_value(builder, &owner.return_type, ptr_ty);
        builder.ins().return_(&[fallback]);
    }
    builder.seal_block(fail_block);

    builder.switch_to_block(pass_block);
    builder.seal_block(pass_block);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_go_stmt(
    expr: &MirExpr,
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &BTreeMap<String, Variable>,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<(), String> {
    let MirExpr::Call { callee, args } = expr else {
        return Err("E3301: unsupported `go` target: expected direct function call".to_string());
    };
    let MirExpr::Var(name) = &**callee else {
        return Err("E3301: unsupported `go` target: expected direct function call".to_string());
    };
    let Some(fid) = function_ids.get(name) else {
        return Err(format!("E3302: unknown go call target `{name}`"));
    };
    let local_target = module.declare_func_in_func(*fid, builder.func);
    let fn_ptr = builder.ins().func_addr(ptr_ty, local_target);

    match args.len() {
        0 => {
            let local_spawn0 = module.declare_func_in_func(runtime_fns.spawn0_fn, builder.func);
            let _ = builder.ins().call(local_spawn0, &[fn_ptr]);
            Ok(())
        }
        1 => {
            let arg = emit_expr(
                &args[0],
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            let arg_ty = builder.func.dfg.value_type(arg);
            let arg_i64 = if arg_ty == ir::types::I64 {
                arg
            } else if arg_ty.is_int() && arg_ty.bits() < 64 {
                builder.ins().sextend(ir::types::I64, arg)
            } else if arg_ty.is_int() && arg_ty.bits() > 64 {
                builder.ins().ireduce(ir::types::I64, arg)
            } else {
                return Err(
                    "E3303: unsupported `go` argument type (expected integer-compatible value)"
                        .to_string(),
                );
            };
            let local_spawn1 = module.declare_func_in_func(runtime_fns.spawn1_i64_fn, builder.func);
            let _ = builder.ins().call(local_spawn1, &[fn_ptr, arg_i64]);
            Ok(())
        }
        n => Err(format!(
            "E3304: unsupported `go` call shape: expected 0 or 1 argument, got {n}"
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_while_stmt(
    cond: &MirExpr,
    body: &[MirStmt],
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &mut BTreeMap<String, Variable>,
    next_var: &mut usize,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<(), String> {
    let header_block = builder.create_block();
    let body_block = builder.create_block();
    let exit_block = builder.create_block();
    builder.ins().jump(header_block, &[]);

    builder.switch_to_block(header_block);
    let cond_v = emit_expr(
        cond,
        module,
        builder,
        locals,
        function_ids,
        function_returns,
        runtime_fns,
        ptr_ty,
        str_data_counter,
        owner,
    )?;
    let cond_ty = builder.func.dfg.value_type(cond_v);
    let zero = builder.ins().iconst(cond_ty, 0);
    let cond_b = builder.ins().icmp(IntCC::NotEqual, cond_v, zero);
    builder.ins().brif(cond_b, body_block, &[], exit_block, &[]);

    builder.switch_to_block(body_block);
    let mut body_terminated = false;
    for stmt in body {
        if body_terminated {
            break;
        }
        body_terminated = emit_stmt(
            stmt,
            module,
            builder,
            locals,
            next_var,
            function_ids,
            function_returns,
            runtime_fns,
            ptr_ty,
            str_data_counter,
            owner,
        )?;
    }
    if !body_terminated {
        builder.ins().jump(header_block, &[]);
    }
    builder.seal_block(body_block);
    builder.seal_block(header_block);

    builder.switch_to_block(exit_block);
    builder.seal_block(exit_block);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_repeat_stmt(
    count: &MirExpr,
    body: &[MirStmt],
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &mut BTreeMap<String, Variable>,
    next_var: &mut usize,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<(), String> {
    let loop_count = emit_expr(
        count,
        module,
        builder,
        locals,
        function_ids,
        function_returns,
        runtime_fns,
        ptr_ty,
        str_data_counter,
        owner,
    )?;
    let idx_var = Variable::from_u32(*next_var as u32);
    *next_var += 1;
    builder.declare_var(idx_var, ir::types::I64);
    let zero = builder.ins().iconst(ir::types::I64, 0);
    builder.def_var(idx_var, zero);

    let header_block = builder.create_block();
    let body_block = builder.create_block();
    let exit_block = builder.create_block();
    builder.ins().jump(header_block, &[]);

    builder.switch_to_block(header_block);
    let idx_val = builder.use_var(idx_var);
    let loop_count_ty = builder.func.dfg.value_type(loop_count);
    let idx_cast = if loop_count_ty == ir::types::I64 {
        idx_val
    } else {
        builder.ins().ireduce(loop_count_ty, idx_val)
    };
    let cond_b = builder
        .ins()
        .icmp(IntCC::SignedLessThan, idx_cast, loop_count);
    builder.ins().brif(cond_b, body_block, &[], exit_block, &[]);

    builder.switch_to_block(body_block);
    let mut body_terminated = false;
    for stmt in body {
        if body_terminated {
            break;
        }
        body_terminated = emit_stmt(
            stmt,
            module,
            builder,
            locals,
            next_var,
            function_ids,
            function_returns,
            runtime_fns,
            ptr_ty,
            str_data_counter,
            owner,
        )?;
    }
    if !body_terminated {
        let one = builder.ins().iconst(ir::types::I64, 1);
        let current_idx = builder.use_var(idx_var);
        let next_idx = builder.ins().iadd(current_idx, one);
        builder.def_var(idx_var, next_idx);
        builder.ins().jump(header_block, &[]);
    }
    builder.seal_block(body_block);
    builder.seal_block(header_block);

    builder.switch_to_block(exit_block);
    builder.seal_block(exit_block);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_select_stmt(
    cases: &[MirSelectCase],
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &mut BTreeMap<String, Variable>,
    next_var: &mut usize,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
) -> Result<(), String> {
    if cases.is_empty() {
        return Ok(());
    }

    let poll_case_indices = cases
        .iter()
        .enumerate()
        .filter_map(|(idx, case)| match case.pattern {
            MirSelectPattern::Receive { .. } | MirSelectPattern::Closed { .. } => Some(idx),
            _ => None,
        })
        .collect::<Vec<_>>();
    let default_case_idx = cases
        .iter()
        .position(|case| matches!(case.pattern, MirSelectPattern::Default));
    let after_case_idx = cases
        .iter()
        .position(|case| matches!(case.pattern, MirSelectPattern::After { .. }));
    let after_duration_ms = after_case_idx.and_then(|idx| match &cases[idx].pattern {
        MirSelectPattern::After { duration_literal } => {
            Some(parse_duration_literal(duration_literal))
        }
        _ => None,
    });

    if poll_case_indices.is_empty() {
        if let Some(default_idx) = default_case_idx {
            let _ = emit_expr(
                &cases[default_idx].action,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            return Ok(());
        }
        if let Some(after_idx) = after_case_idx {
            if let MirSelectPattern::After { duration_literal } = &cases[after_idx].pattern {
                let millis = parse_duration_literal(duration_literal);
                let delay = builder.ins().iconst(ir::types::I64, millis);
                let sleep_local =
                    module.declare_func_in_func(runtime_fns.sleep_ms_fn, builder.func);
                builder.ins().call(sleep_local, &[delay]);
            }
            let _ = emit_expr(
                &cases[after_idx].action,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            return Ok(());
        }
        return Ok(());
    }

    let entry_block = builder.create_block();
    let exit_block = builder.create_block();
    let wait_block = if default_case_idx.is_none() && after_case_idx.is_none() {
        Some(builder.create_block())
    } else {
        None
    };
    let dispatch_blocks = (0..poll_case_indices.len())
        .map(|_| builder.create_block())
        .collect::<Vec<_>>();
    let after_waited_var = after_case_idx.map(|_| {
        let var = Variable::from_u32(*next_var as u32);
        *next_var += 1;
        builder.declare_var(var, ir::types::I64);
        let zero = builder.ins().iconst(ir::types::I64, 0);
        builder.def_var(var, zero);
        var
    });

    builder.ins().jump(entry_block, &[]);
    builder.switch_to_block(entry_block);
    let count_value = builder
        .ins()
        .iconst(ir::types::I64, poll_case_indices.len() as i64);
    let select_cursor_local =
        module.declare_func_in_func(runtime_fns.select_cursor_fn, builder.func);
    let cursor_call = builder.ins().call(select_cursor_local, &[count_value]);
    let cursor_value = builder
        .inst_results(cursor_call)
        .first()
        .copied()
        .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
    let mut switch = Switch::new();
    for (idx, block) in dispatch_blocks.iter().enumerate() {
        switch.set_entry(idx as u128, *block);
    }
    switch.emit(builder, cursor_value, dispatch_blocks[0]);

    for (start_pos, dispatch_block) in dispatch_blocks.iter().enumerate() {
        builder.switch_to_block(*dispatch_block);
        let tail_block = builder.create_block();

        let ordered_case_indices = (0..poll_case_indices.len())
            .map(|offset| {
                let rotated = (start_pos + offset) % poll_case_indices.len();
                poll_case_indices[rotated]
            })
            .collect::<Vec<_>>();

        for (order_idx, case_idx) in ordered_case_indices.iter().enumerate() {
            let case = &cases[*case_idx];
            let continue_block = if order_idx + 1 == ordered_case_indices.len() {
                tail_block
            } else {
                builder.create_block()
            };

            match &case.pattern {
                MirSelectPattern::Receive { binding, source } => {
                    let channel_source = match source {
                        MirExpr::Call { callee, args }
                            if args.is_empty()
                                && matches!(
                                    &**callee,
                                    MirExpr::Member { field, .. } if field == "recv"
                                ) =>
                        {
                            if let MirExpr::Member { object, .. } = &**callee {
                                object.as_ref()
                            } else {
                                source
                            }
                        }
                        _ => source,
                    };
                    let channel = emit_expr(
                        channel_source,
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    let has_data_local =
                        module.declare_func_in_func(runtime_fns.chan_has_data_fn, builder.func);
                    let has_data_call = builder.ins().call(has_data_local, &[channel]);
                    let ready_value = builder
                        .inst_results(has_data_call)
                        .first()
                        .copied()
                        .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
                    let is_ready = builder.ins().icmp_imm(IntCC::NotEqual, ready_value, 0);
                    let ready_block = builder.create_block();
                    builder
                        .ins()
                        .brif(is_ready, ready_block, &[], continue_block, &[]);

                    builder.switch_to_block(ready_block);
                    let recv_local =
                        module.declare_func_in_func(runtime_fns.chan_recv_fn, builder.func);
                    let recv_call = builder.ins().call(recv_local, &[channel]);
                    let recv_value = builder
                        .inst_results(recv_call)
                        .first()
                        .copied()
                        .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
                    let var = if let Some(existing) = locals.get(binding) {
                        *existing
                    } else {
                        let created = Variable::from_u32(*next_var as u32);
                        *next_var += 1;
                        builder.declare_var(created, ir::types::I64);
                        locals.insert(binding.clone(), created);
                        created
                    };
                    builder.def_var(var, recv_value);
                    let _ = emit_expr(
                        &case.action,
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    builder.ins().jump(exit_block, &[]);
                    builder.seal_block(ready_block);
                    builder.switch_to_block(continue_block);
                    builder.seal_block(continue_block);
                }
                MirSelectPattern::Closed { ident } => {
                    let Some(var) = locals.get(ident).copied() else {
                        builder.ins().jump(continue_block, &[]);
                        builder.switch_to_block(continue_block);
                        builder.seal_block(continue_block);
                        continue;
                    };
                    let channel = builder.use_var(var);
                    let closed_local =
                        module.declare_func_in_func(runtime_fns.chan_is_closed_fn, builder.func);
                    let closed_call = builder.ins().call(closed_local, &[channel]);
                    let closed_value = builder
                        .inst_results(closed_call)
                        .first()
                        .copied()
                        .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
                    let is_closed = builder.ins().icmp_imm(IntCC::NotEqual, closed_value, 0);
                    let ready_block = builder.create_block();
                    builder
                        .ins()
                        .brif(is_closed, ready_block, &[], continue_block, &[]);
                    builder.switch_to_block(ready_block);
                    let _ = emit_expr(
                        &case.action,
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    builder.ins().jump(exit_block, &[]);
                    builder.seal_block(ready_block);
                    builder.switch_to_block(continue_block);
                    builder.seal_block(continue_block);
                }
                MirSelectPattern::After { .. } | MirSelectPattern::Default => {
                    builder.ins().jump(continue_block, &[]);
                    builder.switch_to_block(continue_block);
                    builder.seal_block(continue_block);
                }
            }
        }

        builder.switch_to_block(tail_block);
        if let Some(default_idx) = default_case_idx {
            let _ = emit_expr(
                &cases[default_idx].action,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            builder.ins().jump(exit_block, &[]);
        } else if let Some(after_idx) = after_case_idx {
            let waited_var = after_waited_var.expect("after wait variable should exist");
            let waited = builder.use_var(waited_var);
            let needs_sleep = builder.ins().icmp_imm(IntCC::Equal, waited, 0);
            let sleep_block = builder.create_block();
            let after_block = builder.create_block();
            builder
                .ins()
                .brif(needs_sleep, sleep_block, &[], after_block, &[]);

            builder.switch_to_block(sleep_block);
            let one = builder.ins().iconst(ir::types::I64, 1);
            builder.def_var(waited_var, one);
            let delay = builder
                .ins()
                .iconst(ir::types::I64, after_duration_ms.unwrap_or(0));
            let sleep_local = module.declare_func_in_func(runtime_fns.sleep_ms_fn, builder.func);
            builder.ins().call(sleep_local, &[delay]);
            builder.ins().jump(entry_block, &[]);
            builder.seal_block(sleep_block);

            builder.switch_to_block(after_block);
            let _ = emit_expr(
                &cases[after_idx].action,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            builder.ins().jump(exit_block, &[]);
            builder.seal_block(after_block);
        } else if let Some(wait_block) = wait_block {
            builder.ins().jump(wait_block, &[]);
        } else {
            builder.ins().jump(exit_block, &[]);
        }

        builder.seal_block(*dispatch_block);
    }

    if let Some(wait_block) = wait_block {
        builder.switch_to_block(wait_block);
        let delay = builder.ins().iconst(ir::types::I64, 1);
        let sleep_local = module.declare_func_in_func(runtime_fns.sleep_ms_fn, builder.func);
        builder.ins().call(sleep_local, &[delay]);
        builder.ins().jump(entry_block, &[]);
        builder.seal_block(wait_block);
    }

    builder.switch_to_block(exit_block);
    builder.seal_block(entry_block);
    builder.seal_block(exit_block);
    Ok(())
}

fn parse_duration_literal(duration_literal: &str) -> i64 {
    let digits = duration_literal
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>();
    digits.parse::<i64>().unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
fn emit_expr(
    expr: &MirExpr,
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    locals: &BTreeMap<String, Variable>,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
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
        MirExpr::List(items) => {
            let local_new = module.declare_func_in_func(runtime_fns.list_new_i64_fn, builder.func);
            let capacity = builder.ins().iconst(ir::types::I64, items.len() as i64);
            let call = builder.ins().call(local_new, &[capacity]);
            let list_handle = builder
                .inst_results(call)
                .first()
                .copied()
                .ok_or_else(|| "list runtime call did not return list handle".to_string())?;
            for item in items {
                let value = emit_expr(
                    item,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                )?;
                if builder.func.dfg.value_type(value) != ir::types::I64 {
                    return Err(
                        "E3401: native list lowering currently supports List<Int> only".to_string(),
                    );
                }
                let local_append =
                    module.declare_func_in_func(runtime_fns.list_append_i64_fn, builder.func);
                builder.ins().call(local_append, &[list_handle, value]);
            }
            list_handle
        }
        MirExpr::Map(entries) => {
            if entries.is_empty() {
                let local_new =
                    module.declare_func_in_func(runtime_fns.map_new_i64_i64_fn, builder.func);
                let call = builder.ins().call(local_new, &[]);
                builder
                    .inst_results(call)
                    .first()
                    .copied()
                    .ok_or_else(|| "map runtime call did not return map handle".to_string())?
            } else {
                let (first_key_expr, _) = &entries[0];
                let use_str_keys = is_known_string_expr(first_key_expr);
                let local_new = if use_str_keys {
                    module.declare_func_in_func(runtime_fns.map_new_str_i64_fn, builder.func)
                } else {
                    module.declare_func_in_func(runtime_fns.map_new_i64_i64_fn, builder.func)
                };
                let call = builder.ins().call(local_new, &[]);
                let map_handle = builder
                    .inst_results(call)
                    .first()
                    .copied()
                    .ok_or_else(|| "map runtime call did not return map handle".to_string())?;
                for (key_expr, value_expr) in entries {
                    let key = emit_expr(
                        key_expr,
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    let value = emit_expr(
                        value_expr,
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    if builder.func.dfg.value_type(value) != ir::types::I64 {
                        return Err(
                            "E3401: native map lowering currently supports Int values only"
                                .to_string(),
                        );
                    }
                    if use_str_keys {
                        if !is_known_string_expr(key_expr) {
                            return Err(
                                "E3401: map literal key kinds must be consistent (all Str or all Int)"
                                    .to_string(),
                            );
                        }
                        let local_set = module.declare_func_in_func(
                            runtime_fns.container_set_str_i64_fn,
                            builder.func,
                        );
                        builder.ins().call(local_set, &[map_handle, key, value]);
                    } else {
                        if is_known_string_expr(key_expr) {
                            return Err(
                                "E3401: map literal key kinds must be consistent (all Str or all Int)"
                                    .to_string(),
                            );
                        }
                        let local_set = module
                            .declare_func_in_func(runtime_fns.container_set_i64_fn, builder.func);
                        builder.ins().call(local_set, &[map_handle, key, value]);
                    }
                }
                map_handle
            }
        }
        MirExpr::Member { object, field } => {
            let container = emit_expr(
                object,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            if field == "len" {
                let local_len =
                    module.declare_func_in_func(runtime_fns.container_len_fn, builder.func);
                let call = builder.ins().call(local_len, &[container]);
                return Ok(builder
                    .inst_results(call)
                    .first()
                    .copied()
                    .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
            }
            return Err(format!(
                "E3402: member access `{field}` native lowering is not available in v0.1 backend"
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
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    let local_print =
                        module.declare_func_in_func(runtime_fns.print_fn, builder.func);
                    builder.ins().call(local_print, &[arg0]);
                    return Ok(builder.ins().iconst(ir::types::I64, 0));
                }
                if name == "chan" {
                    if args.len() != 1 {
                        return Err("`chan` expects one capacity argument".to_string());
                    }
                    let capacity = emit_expr(
                        &args[0],
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?;
                    let local_new =
                        module.declare_func_in_func(runtime_fns.chan_new_fn, builder.func);
                    let call = builder.ins().call(local_new, &[capacity]);
                    let chan = builder.inst_results(call).first().copied().ok_or_else(|| {
                        "chan runtime call did not return channel handle".to_string()
                    })?;
                    return Ok(chan);
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
                            runtime_fns,
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
                return Err(format!("E3403: unknown call target `{name}`"));
            }
            if let MirExpr::Member { object, field } = &**callee {
                let object_value = emit_expr(
                    object,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                )?;
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(emit_expr(
                        arg,
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                    )?);
                }
                match field.as_str() {
                    "send" => {
                        if lowered_args.len() != 1 {
                            return Err("channel send expects one argument".to_string());
                        }
                        let value = lowered_args[0];
                        let local_send =
                            module.declare_func_in_func(runtime_fns.chan_send_fn, builder.func);
                        let call = builder.ins().call(local_send, &[object_value, value]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "recv" => {
                        if !lowered_args.is_empty() {
                            return Err("channel recv expects no arguments".to_string());
                        }
                        let local_recv =
                            module.declare_func_in_func(runtime_fns.chan_recv_fn, builder.func);
                        let call = builder.ins().call(local_recv, &[object_value]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "close" => {
                        if !lowered_args.is_empty() {
                            return Err("channel close expects no arguments".to_string());
                        }
                        let local_close =
                            module.declare_func_in_func(runtime_fns.chan_close_fn, builder.func);
                        builder.ins().call(local_close, &[object_value]);
                        return Ok(builder.ins().iconst(ir::types::I64, 0));
                    }
                    "append" => {
                        if lowered_args.len() != 1 {
                            return Err("list append expects one argument".to_string());
                        }
                        if builder.func.dfg.value_type(lowered_args[0]) != ir::types::I64 {
                            return Err(
                                "E3404: list append currently supports Int values only".to_string()
                            );
                        }
                        let local_append = module
                            .declare_func_in_func(runtime_fns.list_append_i64_fn, builder.func);
                        let call = builder
                            .ins()
                            .call(local_append, &[object_value, lowered_args[0]]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "len" => {
                        if !lowered_args.is_empty() {
                            return Err("`.len()` expects no arguments".to_string());
                        }
                        let local_len =
                            module.declare_func_in_func(runtime_fns.container_len_fn, builder.func);
                        let call = builder.ins().call(local_len, &[object_value]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "get" => {
                        if lowered_args.len() != 1 {
                            return Err("`.get()` expects one key/index argument".to_string());
                        }
                        let key = lowered_args[0];
                        let key_is_str = is_known_string_expr(&args[0]);
                        let local_get = if key_is_str {
                            module.declare_func_in_func(
                                runtime_fns.container_get_str_i64_fn,
                                builder.func,
                            )
                        } else {
                            module.declare_func_in_func(
                                runtime_fns.container_get_i64_fn,
                                builder.func,
                            )
                        };
                        let call = builder.ins().call(local_get, &[object_value, key]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "set" => {
                        if lowered_args.len() != 2 {
                            return Err(
                                "`.set()` expects key/index and value arguments".to_string()
                            );
                        }
                        let key = lowered_args[0];
                        let value = lowered_args[1];
                        if builder.func.dfg.value_type(value) != ir::types::I64 {
                            return Err(
                                "E3404: `.set()` currently supports Int values only".to_string()
                            );
                        }
                        let key_is_str = is_known_string_expr(&args[0]);
                        let local_set = if key_is_str {
                            module.declare_func_in_func(
                                runtime_fns.container_set_str_i64_fn,
                                builder.func,
                            )
                        } else {
                            module.declare_func_in_func(
                                runtime_fns.container_set_i64_fn,
                                builder.func,
                            )
                        };
                        let call = builder.ins().call(local_set, &[object_value, key, value]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "contains" => {
                        if lowered_args.len() != 1 {
                            return Err("`.contains()` expects one key argument".to_string());
                        }
                        let key = lowered_args[0];
                        let key_is_str = is_known_string_expr(&args[0]);
                        let local_contains = if key_is_str {
                            module.declare_func_in_func(
                                runtime_fns.container_contains_str_i64_fn,
                                builder.func,
                            )
                        } else {
                            module.declare_func_in_func(
                                runtime_fns.container_contains_i64_fn,
                                builder.func,
                            )
                        };
                        let call = builder.ins().call(local_contains, &[object_value, key]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    "remove" => {
                        if lowered_args.len() != 1 {
                            return Err("`.remove()` expects one key argument".to_string());
                        }
                        let key = lowered_args[0];
                        let key_is_str = is_known_string_expr(&args[0]);
                        let local_remove = if key_is_str {
                            module.declare_func_in_func(
                                runtime_fns.container_remove_str_i64_fn,
                                builder.func,
                            )
                        } else {
                            module.declare_func_in_func(
                                runtime_fns.container_remove_i64_fn,
                                builder.func,
                            )
                        };
                        let call = builder.ins().call(local_remove, &[object_value, key]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
                    }
                    _ => {
                        return Err(format!(
                            "E3404: member call `.{field}()` is not supported in v0.1 native backend"
                        ));
                    }
                }
            }
            return Err(
                "E3405: dynamic call targets are not supported in v0.1 native backend".to_string(),
            );
        }
        MirExpr::Binary { left, op, right } => {
            let l = emit_expr(
                left,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
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
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
            )?;
            match op.as_str() {
                "Add" => {
                    if is_known_string_expr(left) && is_known_string_expr(right) {
                        let local_concat =
                            module.declare_func_in_func(runtime_fns.str_concat_fn, builder.func);
                        let call = builder.ins().call(local_concat, &[l, r]);
                        builder.inst_results(call).first().copied().ok_or_else(|| {
                            "string concat runtime call returned no value".to_string()
                        })?
                    } else {
                        builder.ins().iadd(l, r)
                    }
                }
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
                _ => {
                    return Err(format!(
                        "E3406: unsupported binary op `{op}` in native backend"
                    ))
                }
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
                runtime_fns,
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
                _ => {
                    return Err(format!(
                        "E3407: unsupported unary op `{op}` in native backend"
                    ))
                }
            }
        }
        MirExpr::Question { expr } | MirExpr::Old { expr } => emit_expr(
            expr,
            module,
            builder,
            locals,
            function_ids,
            function_returns,
            runtime_fns,
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

fn is_known_string_expr(expr: &MirExpr) -> bool {
    match expr {
        MirExpr::Str(_) => true,
        MirExpr::Binary { left, op, right } if op == "Add" => {
            is_known_string_expr(left) && is_known_string_expr(right)
        }
        _ => false,
    }
}

fn value_type_for_expr(expr: &MirExpr, ptr_ty: ir::Type) -> ir::Type {
    match expr {
        MirExpr::Float(_) => ir::types::F64,
        MirExpr::Bool(_) => ir::types::I8,
        MirExpr::Str(_) => ptr_ty,
        MirExpr::Binary { left, op, right } if op == "Add" => {
            if is_known_string_expr(left) && is_known_string_expr(right) {
                ptr_ty
            } else {
                ir::types::I64
            }
        }
        MirExpr::List(_) | MirExpr::Map(_) => ptr_ty,
        MirExpr::Member { field, .. } if field == "len" => ir::types::I64,
        MirExpr::Call { callee, .. } if matches!(&**callee, MirExpr::Var(name) if name == "chan") => {
            ptr_ty
        }
        MirExpr::Call { callee, .. }
            if matches!(&**callee, MirExpr::Member { field, .. } if field == "get"
                || field == "len"
                || field == "append"
                || field == "set"
                || field == "contains"
                || field == "remove"
                || field == "recv"
                || field == "send"
                || field == "close") =>
        {
            ir::types::I64
        }
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

    #[test]
    fn emits_objects_for_phase6_target_triples() {
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: MirType::I64,
                body: vec![MirStmt::Return(MirExpr::Int(0))],
            }],
        };
        for target in [
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "aarch64-apple-darwin",
        ] {
            let object = emit_object(
                &program,
                &CodegenOptions {
                    target: target.to_string(),
                    ..CodegenOptions::default()
                },
            )
            .expect("target object emission should succeed");
            assert!(!object.is_empty(), "target object should not be empty");
        }
    }
}
