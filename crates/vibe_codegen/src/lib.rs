// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::sync::Arc;

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlags, UserFuncName};
use cranelift_codegen::isa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Switch, Variable};
use cranelift_module::DataDescription;
use cranelift_module::{default_libcall_names, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;
use vibe_mir::{
    MirContractKind, MirExpr, MirForIterKind, MirFunction, MirProgram, MirSelectCase,
    MirSelectPattern, MirStmt, MirType,
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
#[allow(dead_code)]
struct RuntimeFunctions {
    print_fn: FuncId,
    panic_fn: FuncId,
    list_new_i64_fn: FuncId,
    list_append_i64_fn: FuncId,
    list_sort_desc_i64_fn: FuncId,
    list_take_i64_fn: FuncId,
    container_len_fn: FuncId,
    container_get_i64_fn: FuncId,
    container_set_i64_fn: FuncId,
    container_get_auto_i64_fn: FuncId,
    container_set_auto_i64_fn: FuncId,
    container_contains_i64_fn: FuncId,
    container_remove_i64_fn: FuncId,
    map_new_i64_i64_fn: FuncId,
    map_new_str_i64_fn: FuncId,
    container_get_str_i64_fn: FuncId,
    container_set_str_i64_fn: FuncId,
    container_contains_str_i64_fn: FuncId,
    container_remove_str_i64_fn: FuncId,
    map_new_str_str_fn: FuncId,
    container_get_str_str_fn: FuncId,
    container_set_str_str_fn: FuncId,
    json_from_str_str_map_fn: FuncId,
    container_contains_auto_i64_fn: FuncId,
    container_remove_auto_i64_fn: FuncId,
    container_key_at_i64_fn: FuncId,
    container_key_at_str_fn: FuncId,
    container_eq_fn: FuncId,
    str_len_bytes_fn: FuncId,
    str_get_byte_fn: FuncId,
    str_slice_fn: FuncId,
    str_eq_fn: FuncId,
    str_concat_fn: FuncId,
    chan_new_fn: FuncId,
    chan_send_fn: FuncId,
    chan_recv_fn: FuncId,
    chan_has_data_fn: FuncId,
    chan_close_fn: FuncId,
    chan_is_closed_fn: FuncId,
    spawn0_fn: FuncId,
    spawn1_i64_fn: FuncId,
    async_i64_fn: FuncId,
    async_ptr_fn: FuncId,
    await_i64_fn: FuncId,
    await_ptr_fn: FuncId,
    select_cursor_fn: FuncId,
    sleep_ms_fn: FuncId,
    json_encode_record_fn: FuncId,
    json_decode_record_fn: FuncId,
    json_builder_new_fn: FuncId,
    json_builder_begin_object_fn: FuncId,
    json_builder_end_object_fn: FuncId,
    json_builder_begin_array_fn: FuncId,
    json_builder_end_array_fn: FuncId,
    json_builder_key_fn: FuncId,
    json_builder_value_null_fn: FuncId,
    json_builder_value_bool_fn: FuncId,
    json_builder_value_i64_fn: FuncId,
    json_builder_value_f64_fn: FuncId,
    json_builder_value_str_fn: FuncId,
    json_builder_value_json_fn: FuncId,
    json_builder_finish_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_md5_hex_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_md5_bytes_hex_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_json_canonical_fn: FuncId,

    #[cfg(feature = "bench-runtime")]
    bench_json_repeat_array_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_http_server_bench_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_secp256k1_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_edigits_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_listen_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_listener_port_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_accept_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_connect_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_read_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_write_fn: FuncId,
    #[cfg(feature = "bench-runtime")]
    bench_net_close_fn: FuncId,
    record_alloc_fn: FuncId,
}

#[derive(Clone, Copy)]
struct LoopContext {
    break_block: ir::Block,
    continue_block: ir::Block,
}

pub fn emit_object(program: &MirProgram, options: &CodegenOptions) -> Result<Vec<u8>, String> {
    emit_object_with_types(
        program,
        options,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

#[allow(unused_variables)]
pub fn emit_object_with_types(
    program: &MirProgram,
    options: &CodegenOptions,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
    namespace_map: &BTreeMap<(String, String), String>,
) -> Result<Vec<u8>, String> {
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

    for ((ns, field), mangled) in namespace_map {
        let ns_key = format!("__ns_call__{ns}.{field}");
        if let Some(&fid) = function_ids.get(mangled) {
            function_ids.insert(ns_key.clone(), fid);
        }
        if let Some(ret_ty) = function_returns.get(mangled).cloned() {
            function_returns.insert(ns_key, ret_ty);
        }
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
            type_defs,
            enum_defs,
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

    let mut list_sort_desc_sig = module.make_signature();
    list_sort_desc_sig.params.push(AbiParam::new(ptr_ty));
    list_sort_desc_sig.returns.push(AbiParam::new(ptr_ty));
    let list_sort_desc_i64_fn = module
        .declare_function(
            "vibe_list_sort_desc_i64",
            Linkage::Import,
            &list_sort_desc_sig,
        )
        .map_err(|e| format!("failed to declare runtime list_sort_desc symbol: {e}"))?;

    let mut list_take_sig = module.make_signature();
    list_take_sig.params.push(AbiParam::new(ptr_ty));
    list_take_sig.params.push(AbiParam::new(ir::types::I64));
    list_take_sig.returns.push(AbiParam::new(ptr_ty));
    let list_take_i64_fn = module
        .declare_function("vibe_list_take_i64", Linkage::Import, &list_take_sig)
        .map_err(|e| format!("failed to declare runtime list_take symbol: {e}"))?;

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

    let mut map_new_str_str_sig = module.make_signature();
    map_new_str_str_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    map_new_str_str_sig.returns.push(AbiParam::new(ptr_ty));
    let map_new_str_str_fn = module
        .declare_function(
            "vibe_map_new_str_str",
            Linkage::Import,
            &map_new_str_str_sig,
        )
        .map_err(|e| format!("failed to declare runtime map_new_str_str symbol: {e}"))?;

    let mut container_get_str_str_sig = module.make_signature();
    container_get_str_str_sig.params.push(AbiParam::new(ptr_ty));
    container_get_str_str_sig.params.push(AbiParam::new(ptr_ty));
    container_get_str_str_sig
        .returns
        .push(AbiParam::new(ptr_ty));
    let container_get_str_str_fn = module
        .declare_function(
            "vibe_map_get_str_str",
            Linkage::Import,
            &container_get_str_str_sig,
        )
        .map_err(|e| format!("failed to declare runtime map_get_str_str symbol: {e}"))?;

    let mut container_set_str_str_sig = module.make_signature();
    container_set_str_str_sig.params.push(AbiParam::new(ptr_ty));
    container_set_str_str_sig.params.push(AbiParam::new(ptr_ty));
    container_set_str_str_sig.params.push(AbiParam::new(ptr_ty));
    let container_set_str_str_fn = module
        .declare_function(
            "vibe_map_set_str_str",
            Linkage::Import,
            &container_set_str_str_sig,
        )
        .map_err(|e| format!("failed to declare runtime map_set_str_str symbol: {e}"))?;

    let mut json_from_str_str_map_sig = module.make_signature();
    json_from_str_str_map_sig.params.push(AbiParam::new(ptr_ty));
    json_from_str_str_map_sig
        .returns
        .push(AbiParam::new(ptr_ty));
    let json_from_str_str_map_fn = module
        .declare_function(
            "vibe_json_from_str_str_map",
            Linkage::Import,
            &json_from_str_str_map_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_from_str_str_map symbol: {e}"))?;

    let mut str_concat_sig = module.make_signature();
    str_concat_sig.params.push(AbiParam::new(ptr_ty));
    str_concat_sig.params.push(AbiParam::new(ptr_ty));
    str_concat_sig.returns.push(AbiParam::new(ptr_ty));
    let str_concat_fn = module
        .declare_function("vibe_str_concat", Linkage::Import, &str_concat_sig)
        .map_err(|e| format!("failed to declare runtime str_concat symbol: {e}"))?;

    let mut str_len_bytes_sig = module.make_signature();
    str_len_bytes_sig.params.push(AbiParam::new(ptr_ty));
    str_len_bytes_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let str_len_bytes_fn = module
        .declare_function("vibe_str_len_bytes", Linkage::Import, &str_len_bytes_sig)
        .map_err(|e| format!("failed to declare runtime str_len_bytes symbol: {e}"))?;

    let mut str_get_byte_sig = module.make_signature();
    str_get_byte_sig.params.push(AbiParam::new(ptr_ty));
    str_get_byte_sig.params.push(AbiParam::new(ir::types::I64));
    str_get_byte_sig.returns.push(AbiParam::new(ir::types::I64));
    let str_get_byte_fn = module
        .declare_function("vibe_str_get_byte", Linkage::Import, &str_get_byte_sig)
        .map_err(|e| format!("failed to declare runtime str_get_byte symbol: {e}"))?;

    let mut str_slice_sig = module.make_signature();
    str_slice_sig.params.push(AbiParam::new(ptr_ty));
    str_slice_sig.params.push(AbiParam::new(ir::types::I64));
    str_slice_sig.params.push(AbiParam::new(ir::types::I64));
    str_slice_sig.returns.push(AbiParam::new(ptr_ty));
    let str_slice_fn = module
        .declare_function("vibe_str_slice", Linkage::Import, &str_slice_sig)
        .map_err(|e| format!("failed to declare runtime str_slice symbol: {e}"))?;

    let mut str_eq_sig = module.make_signature();
    str_eq_sig.params.push(AbiParam::new(ptr_ty));
    str_eq_sig.params.push(AbiParam::new(ptr_ty));
    str_eq_sig.returns.push(AbiParam::new(ir::types::I64));
    let str_eq_fn = module
        .declare_function("vibe_str_eq", Linkage::Import, &str_eq_sig)
        .map_err(|e| format!("failed to declare runtime str_eq symbol: {e}"))?;

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

    let mut container_get_auto_i64_sig = module.make_signature();
    container_get_auto_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_get_auto_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_get_auto_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_get_auto_i64_fn = module
        .declare_function(
            "vibe_container_get_auto_i64",
            Linkage::Import,
            &container_get_auto_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_get_auto_i64 symbol: {e}"))?;

    let mut container_set_auto_i64_sig = module.make_signature();
    container_set_auto_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_set_auto_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_set_auto_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_set_auto_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_set_auto_i64_fn = module
        .declare_function(
            "vibe_container_set_auto_i64",
            Linkage::Import,
            &container_set_auto_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_set_auto_i64 symbol: {e}"))?;

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

    let mut container_contains_auto_i64_sig = module.make_signature();
    container_contains_auto_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_contains_auto_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_contains_auto_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_contains_auto_i64_fn = module
        .declare_function(
            "vibe_container_contains_auto_i64",
            Linkage::Import,
            &container_contains_auto_i64_sig,
        )
        .map_err(|e| {
            format!("failed to declare runtime container_contains_auto_i64 symbol: {e}")
        })?;

    let mut container_remove_auto_i64_sig = module.make_signature();
    container_remove_auto_i64_sig
        .params
        .push(AbiParam::new(ptr_ty));
    container_remove_auto_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_remove_auto_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_remove_auto_i64_fn = module
        .declare_function(
            "vibe_container_remove_auto_i64",
            Linkage::Import,
            &container_remove_auto_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_remove_auto_i64 symbol: {e}"))?;

    let mut container_key_at_i64_sig = module.make_signature();
    container_key_at_i64_sig.params.push(AbiParam::new(ptr_ty));
    container_key_at_i64_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_key_at_i64_sig
        .returns
        .push(AbiParam::new(ir::types::I64));
    let container_key_at_i64_fn = module
        .declare_function(
            "vibe_container_key_at_i64",
            Linkage::Import,
            &container_key_at_i64_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_key_at_i64 symbol: {e}"))?;

    let mut container_key_at_str_sig = module.make_signature();
    container_key_at_str_sig.params.push(AbiParam::new(ptr_ty));
    container_key_at_str_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    container_key_at_str_sig.returns.push(AbiParam::new(ptr_ty));
    let container_key_at_str_fn = module
        .declare_function(
            "vibe_container_key_at_str",
            Linkage::Import,
            &container_key_at_str_sig,
        )
        .map_err(|e| format!("failed to declare runtime container_key_at_str symbol: {e}"))?;

    let mut container_eq_sig = module.make_signature();
    container_eq_sig.params.push(AbiParam::new(ptr_ty));
    container_eq_sig.params.push(AbiParam::new(ptr_ty));
    container_eq_sig.returns.push(AbiParam::new(ir::types::I64));
    let container_eq_fn = module
        .declare_function("vibe_container_eq", Linkage::Import, &container_eq_sig)
        .map_err(|e| format!("failed to declare runtime container_eq symbol: {e}"))?;

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

    let mut async_i64_sig = module.make_signature();
    async_i64_sig.params.push(AbiParam::new(ir::types::I64));
    async_i64_sig.returns.push(AbiParam::new(ir::types::I64));
    let async_i64_fn = module
        .declare_function("vibe_async_i64", Linkage::Import, &async_i64_sig)
        .map_err(|e| format!("failed to declare runtime async_i64 symbol: {e}"))?;

    let mut async_ptr_sig = module.make_signature();
    async_ptr_sig.params.push(AbiParam::new(ptr_ty));
    async_ptr_sig.returns.push(AbiParam::new(ptr_ty));
    let async_ptr_fn = module
        .declare_function("vibe_async_ptr", Linkage::Import, &async_ptr_sig)
        .map_err(|e| format!("failed to declare runtime async_ptr symbol: {e}"))?;

    let mut await_i64_sig = module.make_signature();
    await_i64_sig.params.push(AbiParam::new(ir::types::I64));
    await_i64_sig.returns.push(AbiParam::new(ir::types::I64));
    let await_i64_fn = module
        .declare_function("vibe_await_i64", Linkage::Import, &await_i64_sig)
        .map_err(|e| format!("failed to declare runtime await_i64 symbol: {e}"))?;

    let mut await_ptr_sig = module.make_signature();
    await_ptr_sig.params.push(AbiParam::new(ptr_ty));
    await_ptr_sig.returns.push(AbiParam::new(ptr_ty));
    let await_ptr_fn = module
        .declare_function("vibe_await_ptr", Linkage::Import, &await_ptr_sig)
        .map_err(|e| format!("failed to declare runtime await_ptr symbol: {e}"))?;

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

    let mut json_encode_record_sig = module.make_signature();
    json_encode_record_sig.params.push(AbiParam::new(ptr_ty));
    json_encode_record_sig.params.push(AbiParam::new(ptr_ty));
    json_encode_record_sig.returns.push(AbiParam::new(ptr_ty));
    let json_encode_record_fn = module
        .declare_function(
            "vibe_json_encode_record",
            Linkage::Import,
            &json_encode_record_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_encode_record symbol: {e}"))?;

    let mut json_decode_record_sig = module.make_signature();
    json_decode_record_sig.params.push(AbiParam::new(ptr_ty)); // raw JSON
    json_decode_record_sig.params.push(AbiParam::new(ptr_ty)); // schema
    json_decode_record_sig.params.push(AbiParam::new(ptr_ty)); // fallback record
    json_decode_record_sig.params.push(AbiParam::new(ptr_ty)); // out record
    json_decode_record_sig.returns.push(AbiParam::new(ptr_ty));
    let json_decode_record_fn = module
        .declare_function(
            "vibe_json_decode_record",
            Linkage::Import,
            &json_decode_record_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_decode_record symbol: {e}"))?;

    let mut json_builder_new_sig = module.make_signature();
    json_builder_new_sig
        .params
        .push(AbiParam::new(ir::types::I64));
    json_builder_new_sig.returns.push(AbiParam::new(ptr_ty));
    let json_builder_new_fn = module
        .declare_function(
            "vibe_json_builder_new",
            Linkage::Import,
            &json_builder_new_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_new symbol: {e}"))?;

    let mut json_builder_one_ptr_sig = module.make_signature();
    json_builder_one_ptr_sig.params.push(AbiParam::new(ptr_ty));
    json_builder_one_ptr_sig.returns.push(AbiParam::new(ptr_ty));
    let json_builder_begin_object_fn = module
        .declare_function(
            "vibe_json_builder_begin_object",
            Linkage::Import,
            &json_builder_one_ptr_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_begin_object symbol: {e}"))?;
    let json_builder_end_object_fn = module
        .declare_function(
            "vibe_json_builder_end_object",
            Linkage::Import,
            &json_builder_one_ptr_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_end_object symbol: {e}"))?;
    let json_builder_begin_array_fn = module
        .declare_function(
            "vibe_json_builder_begin_array",
            Linkage::Import,
            &json_builder_one_ptr_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_begin_array symbol: {e}"))?;
    let json_builder_end_array_fn = module
        .declare_function(
            "vibe_json_builder_end_array",
            Linkage::Import,
            &json_builder_one_ptr_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_end_array symbol: {e}"))?;
    let json_builder_value_null_fn = module
        .declare_function(
            "vibe_json_builder_value_null",
            Linkage::Import,
            &json_builder_one_ptr_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_value_null symbol: {e}"))?;
    let json_builder_finish_fn = module
        .declare_function(
            "vibe_json_builder_finish",
            Linkage::Import,
            &json_builder_one_ptr_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_finish symbol: {e}"))?;

    let mut json_builder_key_sig = module.make_signature();
    json_builder_key_sig.params.push(AbiParam::new(ptr_ty));
    json_builder_key_sig.params.push(AbiParam::new(ptr_ty));
    json_builder_key_sig.returns.push(AbiParam::new(ptr_ty));
    let json_builder_key_fn = module
        .declare_function(
            "vibe_json_builder_key",
            Linkage::Import,
            &json_builder_key_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_key symbol: {e}"))?;
    let json_builder_value_bool_fn = module
        .declare_function("vibe_json_builder_value_bool", Linkage::Import, &{
            let mut sig = module.make_signature();
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ir::types::I64));
            sig.returns.push(AbiParam::new(ptr_ty));
            sig
        })
        .map_err(|e| format!("failed to declare runtime json_builder_value_bool symbol: {e}"))?;
    let json_builder_value_i64_fn = module
        .declare_function("vibe_json_builder_value_i64", Linkage::Import, &{
            let mut sig = module.make_signature();
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ir::types::I64));
            sig.returns.push(AbiParam::new(ptr_ty));
            sig
        })
        .map_err(|e| format!("failed to declare runtime json_builder_value_i64 symbol: {e}"))?;
    let json_builder_value_f64_fn = module
        .declare_function("vibe_json_builder_value_f64", Linkage::Import, &{
            let mut sig = module.make_signature();
            sig.params.push(AbiParam::new(ptr_ty));
            sig.params.push(AbiParam::new(ir::types::F64));
            sig.returns.push(AbiParam::new(ptr_ty));
            sig
        })
        .map_err(|e| format!("failed to declare runtime json_builder_value_f64 symbol: {e}"))?;
    let json_builder_value_str_fn = module
        .declare_function(
            "vibe_json_builder_value_str",
            Linkage::Import,
            &json_builder_key_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_value_str symbol: {e}"))?;
    let json_builder_value_json_fn = module
        .declare_function(
            "vibe_json_builder_value_json",
            Linkage::Import,
            &json_builder_key_sig,
        )
        .map_err(|e| format!("failed to declare runtime json_builder_value_json symbol: {e}"))?;

    #[cfg(feature = "bench-runtime")]
    let bench_md5_hex_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_md5_hex", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime md5_hex symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_md5_bytes_hex_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_md5_bytes_hex", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime md5_bytes_hex symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_json_canonical_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_json_canonical", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime json_canonical symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_json_repeat_array_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_json_repeat_array", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime json_repeat_array symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_http_server_bench_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_http_server_bench", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime http_server_bench symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_secp256k1_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_secp256k1", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime secp256k1 symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_edigits_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_edigits", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime edigits symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_listen_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_net_listen", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_listen symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_listener_port_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_net_listener_port", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_listener_port symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_accept_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_net_accept", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_accept symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_connect_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_net_connect", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_connect symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_read_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ptr_ty));
        module
            .declare_function("vibe_bench_net_read", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_read symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_write_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.params.push(AbiParam::new(ptr_ty));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_net_write", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_write symbol: {e}"))?
    };

    #[cfg(feature = "bench-runtime")]
    let bench_net_close_fn = {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(ir::types::I64));
        sig.returns.push(AbiParam::new(ir::types::I64));
        module
            .declare_function("vibe_bench_net_close", Linkage::Import, &sig)
            .map_err(|e| format!("failed to declare bench runtime net_close symbol: {e}"))?
    };

    let mut record_alloc_sig = module.make_signature();
    record_alloc_sig.params.push(AbiParam::new(ir::types::I64));
    record_alloc_sig.returns.push(AbiParam::new(ptr_ty));
    let record_alloc_fn = module
        .declare_function("vibe_record_alloc", Linkage::Import, &record_alloc_sig)
        .map_err(|e| format!("failed to declare runtime vibe_record_alloc symbol: {e}"))?;

    Ok(RuntimeFunctions {
        print_fn,
        panic_fn,
        list_new_i64_fn,
        list_append_i64_fn,
        list_sort_desc_i64_fn,
        list_take_i64_fn,
        container_len_fn,
        container_get_i64_fn,
        container_set_i64_fn,
        container_get_auto_i64_fn,
        container_set_auto_i64_fn,
        container_contains_i64_fn,
        container_remove_i64_fn,
        map_new_i64_i64_fn,
        map_new_str_i64_fn,
        map_new_str_str_fn,
        container_get_str_i64_fn,
        container_set_str_i64_fn,
        container_contains_str_i64_fn,
        container_get_str_str_fn,
        container_set_str_str_fn,
        json_from_str_str_map_fn,
        container_remove_str_i64_fn,
        container_contains_auto_i64_fn,
        container_remove_auto_i64_fn,
        container_key_at_i64_fn,
        container_key_at_str_fn,
        container_eq_fn,
        str_len_bytes_fn,
        str_get_byte_fn,
        str_slice_fn,
        str_eq_fn,
        str_concat_fn,
        chan_new_fn,
        chan_send_fn,
        chan_recv_fn,
        chan_has_data_fn,
        chan_close_fn,
        chan_is_closed_fn,
        spawn0_fn,
        spawn1_i64_fn,
        async_i64_fn,
        async_ptr_fn,
        await_i64_fn,
        await_ptr_fn,
        select_cursor_fn,
        sleep_ms_fn,
        json_encode_record_fn,
        json_decode_record_fn,
        json_builder_new_fn,
        json_builder_begin_object_fn,
        json_builder_end_object_fn,
        json_builder_begin_array_fn,
        json_builder_end_array_fn,
        json_builder_key_fn,
        json_builder_value_null_fn,
        json_builder_value_bool_fn,
        json_builder_value_i64_fn,
        json_builder_value_f64_fn,
        json_builder_value_str_fn,
        json_builder_value_json_fn,
        json_builder_finish_fn,
        #[cfg(feature = "bench-runtime")]
        bench_md5_hex_fn,
        #[cfg(feature = "bench-runtime")]
        bench_md5_bytes_hex_fn,
        #[cfg(feature = "bench-runtime")]
        bench_json_canonical_fn,
        #[cfg(feature = "bench-runtime")]
        bench_json_repeat_array_fn,
        #[cfg(feature = "bench-runtime")]
        bench_http_server_bench_fn,
        #[cfg(feature = "bench-runtime")]
        bench_secp256k1_fn,
        #[cfg(feature = "bench-runtime")]
        bench_edigits_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_listen_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_listener_port_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_accept_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_connect_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_read_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_write_fn,
        #[cfg(feature = "bench-runtime")]
        bench_net_close_fn,
        record_alloc_fn,
    })
}

#[allow(clippy::too_many_arguments)]
fn define_function(
    module: &mut ObjectModule,
    function: &MirFunction,
    func_id: FuncId,
    function_ids: &BTreeMap<String, FuncId>,
    function_returns: &BTreeMap<String, MirType>,
    runtime_fns: RuntimeFunctions,
    ptr_ty: ir::Type,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
) -> Result<(), String> {
    if let Some(native_sym) = &function.native_symbol {
        let mut ctx = module.make_context();
        let sig = build_signature(module, function, ptr_ty);
        ctx.func.signature = sig.clone();
        ctx.func.name = UserFuncName::user(0, func_id.as_u32());

        let mut native_sig = module.make_signature();
        for p in &function.params {
            native_sig
                .params
                .push(AbiParam::new(mir_ty_to_clif(&p.ty, ptr_ty)));
        }
        if function.return_type != MirType::Void {
            native_sig
                .returns
                .push(AbiParam::new(mir_ty_to_clif(&function.return_type, ptr_ty)));
        }
        let native_id = module
            .declare_function(native_sym, Linkage::Import, &native_sig)
            .map_err(|e| format!("failed to declare native symbol `{native_sym}`: {e}"))?;

        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let local_fn = module.declare_func_in_func(native_id, builder.func);
        let params: Vec<ir::Value> = (0..function.params.len())
            .map(|i| builder.block_params(entry)[i])
            .collect();
        let call = builder.ins().call(local_fn, &params);
        if function.return_type == MirType::Void {
            builder.ins().return_(&[]);
        } else {
            let ret =
                builder.inst_results(call).first().copied().ok_or_else(|| {
                    format!("native function `{native_sym}` did not return a value")
                })?;
            builder.ins().return_(&[ret]);
        }
        builder.finalize();
        module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("failed to define native wrapper `{}`: {e}", function.name))?;
        return Ok(());
    }

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
    let mut locals_ty: BTreeMap<String, MirType> = BTreeMap::new();
    for param in &function.params {
        locals_ty.insert(param.name.clone(), param.ty.clone());
    }
    collect_local_types(&function.body, function, function_returns, &mut locals_ty);

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
            None,
            type_defs,
            enum_defs,
            &locals_ty,
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
    loop_ctx: Option<LoopContext>,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
    locals_ty: &BTreeMap<String, MirType>,
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
                type_defs,
                enum_defs,
            )?;
            let var = Variable::from_u32(*next_var as u32);
            *next_var += 1;
            builder.declare_var(
                var,
                value_type_for_expr(expr, owner, function_returns, ptr_ty, locals_ty),
            );
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
                type_defs,
                enum_defs,
            )?;
            let var = if let Some(v) = locals.get(name) {
                *v
            } else {
                let v = Variable::from_u32(*next_var as u32);
                *next_var += 1;
                builder.declare_var(
                    v,
                    value_type_for_expr(expr, owner, function_returns, ptr_ty, locals_ty),
                );
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
                type_defs,
                enum_defs,
            )?;
            Ok(false)
        }
        MirStmt::Thread(expr) => {
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
            )?;
            if owner.return_type == MirType::Void {
                builder.ins().return_(&[]);
            } else {
                let ret_ty = mir_ty_to_clif(&owner.return_type, ptr_ty);
                let val_ty = builder.func.dfg.value_type(value);
                let coerced = if val_ty == ret_ty {
                    value
                } else if val_ty.is_int() && ret_ty.is_int() && val_ty.bits() > ret_ty.bits() {
                    builder.ins().ireduce(ret_ty, value)
                } else if val_ty.is_int() && ret_ty.is_int() && val_ty.bits() < ret_ty.bits() {
                    builder.ins().uextend(ret_ty, value)
                } else {
                    value
                };
                builder.ins().return_(&[coerced]);
            }
            Ok(true)
        }
        MirStmt::Break => {
            let Some(loop_ctx) = loop_ctx else {
                return Err(
                    "E3410: `break` cannot be emitted outside `for`, `while`, or `repeat`"
                        .to_string(),
                );
            };
            builder.ins().jump(loop_ctx.break_block, &[]);
            Ok(true)
        }
        MirStmt::Continue => {
            let Some(loop_ctx) = loop_ctx else {
                return Err(
                    "E3411: `continue` cannot be emitted outside `for`, `while`, or `repeat`"
                        .to_string(),
                );
            };
            builder.ins().jump(loop_ctx.continue_block, &[]);
            Ok(true)
        }
        MirStmt::For {
            var,
            iter,
            iter_kind,
            body,
        } => {
            emit_for_stmt(
                var,
                iter,
                iter_kind,
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
                type_defs,
                enum_defs,
                locals_ty,
            )?;
            Ok(false)
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
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
                    loop_ctx,
                    type_defs,
                    enum_defs,
                    locals_ty,
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
                    loop_ctx,
                    type_defs,
                    enum_defs,
                    locals_ty,
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
                locals_ty,
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
                type_defs,
                enum_defs,
                locals_ty,
            )?;
            Ok(false)
        }
        MirStmt::Match {
            scrutinee,
            arms,
            default_action,
        } => {
            let scrut_v = emit_expr(
                scrutinee,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
                type_defs,
                enum_defs,
            )?;
            let merge_block = builder.create_block();
            let mut arm_blocks: Vec<ir::Block> = Vec::with_capacity(arms.len());
            for _ in arms.iter() {
                arm_blocks.push(builder.create_block());
            }
            let default_block = builder.create_block();
            let mut check_blocks: Vec<ir::Block> = Vec::with_capacity(arms.len() + 1);
            for _ in 0..=arms.len() {
                check_blocks.push(builder.create_block());
            }
            builder.ins().jump(check_blocks[0], &[]);
            for (i, arm) in arms.iter().enumerate() {
                builder.switch_to_block(check_blocks[i]);
                let MirExpr::EnumVariant { enum_name, variant } = &arm.pattern else {
                    return Err("E3499: match arm pattern must be enum variant".to_string());
                };
                let variants = enum_defs
                    .get(enum_name)
                    .ok_or_else(|| format!("E3499: unknown enum in match arm `{enum_name}`"))?;
                let tag = variants.iter().position(|v| v == variant).unwrap_or(0) as i64;
                let tag_const = builder.ins().iconst(ir::types::I64, tag);
                let cond = builder.ins().icmp(IntCC::Equal, scrut_v, tag_const);
                let next_check = check_blocks[i + 1];
                builder
                    .ins()
                    .brif(cond, arm_blocks[i], &[], next_check, &[]);
            }
            builder.switch_to_block(check_blocks[arms.len()]);
            builder.ins().jump(default_block, &[]);
            builder.switch_to_block(default_block);
            if let Some(default_expr) = default_action {
                let _ = emit_expr(
                    default_expr,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                    type_defs,
                    enum_defs,
                )?;
            }
            builder.ins().jump(merge_block, &[]);
            for (i, arm) in arms.iter().enumerate() {
                builder.switch_to_block(arm_blocks[i]);
                let _ = emit_expr(
                    &arm.action,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                    type_defs,
                    enum_defs,
                )?;
                builder.ins().jump(merge_block, &[]);
            }
            for b in &check_blocks {
                builder.seal_block(*b);
            }
            builder.seal_block(default_block);
            for b in &arm_blocks {
                builder.seal_block(*b);
            }
            builder.switch_to_block(merge_block);
            builder.seal_block(merge_block);
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
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
        type_defs,
        enum_defs,
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
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
                type_defs,
                enum_defs,
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
fn emit_for_stmt(
    var: &str,
    iter: &MirExpr,
    iter_kind: &MirForIterKind,
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
    locals_ty: &BTreeMap<String, MirType>,
) -> Result<(), String> {
    if matches!(iter_kind, MirForIterKind::Unknown) {
        return Err(
            "E3408: native `for in` lowering requires List<T> or Map<K,V> iterable input"
                .to_string(),
        );
    }
    let iter_handle = emit_expr(
        iter,
        module,
        builder,
        locals,
        function_ids,
        function_returns,
        runtime_fns,
        ptr_ty,
        str_data_counter,
        owner,
        type_defs,
        enum_defs,
    )?;
    if builder.func.dfg.value_type(iter_handle) != ptr_ty {
        return Err(
            "E3408: `for in` expects a container expression (List<T> or Map<K,V>)".to_string(),
        );
    }

    let iter_var = Variable::from_u32(*next_var as u32);
    *next_var += 1;
    builder.declare_var(iter_var, ptr_ty);
    builder.def_var(iter_var, iter_handle);

    let loop_var = Variable::from_u32(*next_var as u32);
    *next_var += 1;
    let loop_var_ty = if matches!(iter_kind, MirForIterKind::MapStr) {
        ptr_ty
    } else {
        ir::types::I64
    };
    builder.declare_var(loop_var, loop_var_ty);
    let zero = builder.ins().iconst(ir::types::I64, 0);
    if loop_var_ty == ptr_ty {
        let null_ptr = builder.ins().iconst(ptr_ty, 0);
        builder.def_var(loop_var, null_ptr);
    } else {
        builder.def_var(loop_var, zero);
    }
    locals.insert(var.to_string(), loop_var);

    let idx_var = Variable::from_u32(*next_var as u32);
    *next_var += 1;
    builder.declare_var(idx_var, ir::types::I64);
    builder.def_var(idx_var, zero);

    let header_block = builder.create_block();
    let bind_block = builder.create_block();
    let body_block = builder.create_block();
    let continue_block = builder.create_block();
    let exit_block = builder.create_block();
    builder.ins().jump(header_block, &[]);

    builder.switch_to_block(header_block);
    let iter_handle_value = builder.use_var(iter_var);
    let local_len = module.declare_func_in_func(runtime_fns.container_len_fn, builder.func);
    let len_call = builder.ins().call(local_len, &[iter_handle_value]);
    let len_value = builder
        .inst_results(len_call)
        .first()
        .copied()
        .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
    let idx_value = builder.use_var(idx_var);
    let cond = builder
        .ins()
        .icmp(IntCC::SignedLessThan, idx_value, len_value);
    builder.ins().brif(cond, bind_block, &[], exit_block, &[]);

    builder.switch_to_block(bind_block);
    match iter_kind {
        MirForIterKind::List => {
            let local_get =
                module.declare_func_in_func(runtime_fns.container_get_i64_fn, builder.func);
            let call = builder
                .ins()
                .call(local_get, &[iter_handle_value, idx_value]);
            let value = builder
                .inst_results(call)
                .first()
                .copied()
                .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
            builder.def_var(loop_var, value);
        }
        MirForIterKind::MapInt => {
            let local_key =
                module.declare_func_in_func(runtime_fns.container_key_at_i64_fn, builder.func);
            let call = builder
                .ins()
                .call(local_key, &[iter_handle_value, idx_value]);
            let key = builder
                .inst_results(call)
                .first()
                .copied()
                .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0));
            builder.def_var(loop_var, key);
        }
        MirForIterKind::MapStr => {
            let local_key =
                module.declare_func_in_func(runtime_fns.container_key_at_str_fn, builder.func);
            let call = builder
                .ins()
                .call(local_key, &[iter_handle_value, idx_value]);
            let key = builder
                .inst_results(call)
                .first()
                .copied()
                .unwrap_or_else(|| builder.ins().iconst(ptr_ty, 0));
            builder.def_var(loop_var, key);
        }
        MirForIterKind::Unknown => unreachable!("unknown iterator kind should return early"),
    }
    builder.ins().jump(body_block, &[]);

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
            Some(LoopContext {
                break_block: exit_block,
                continue_block,
            }),
            type_defs,
            enum_defs,
            locals_ty,
        )?;
    }
    if !body_terminated {
        builder.ins().jump(continue_block, &[]);
    }

    builder.switch_to_block(continue_block);
    let current_idx = builder.use_var(idx_var);
    let one = builder.ins().iconst(ir::types::I64, 1);
    let next_idx = builder.ins().iadd(current_idx, one);
    builder.def_var(idx_var, next_idx);
    builder.ins().jump(header_block, &[]);

    builder.seal_block(bind_block);
    builder.seal_block(body_block);
    builder.seal_block(continue_block);
    builder.seal_block(header_block);

    builder.switch_to_block(exit_block);
    builder.seal_block(exit_block);
    Ok(())
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
    locals_ty: &BTreeMap<String, MirType>,
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
        type_defs,
        enum_defs,
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
            Some(LoopContext {
                break_block: exit_block,
                continue_block: header_block,
            }),
            type_defs,
            enum_defs,
            locals_ty,
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
    locals_ty: &BTreeMap<String, MirType>,
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
        type_defs,
        enum_defs,
    )?;
    let idx_var = Variable::from_u32(*next_var as u32);
    *next_var += 1;
    builder.declare_var(idx_var, ir::types::I64);
    let zero = builder.ins().iconst(ir::types::I64, 0);
    builder.def_var(idx_var, zero);

    let header_block = builder.create_block();
    let body_block = builder.create_block();
    let continue_block = builder.create_block();
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
            Some(LoopContext {
                break_block: exit_block,
                continue_block,
            }),
            type_defs,
            enum_defs,
            locals_ty,
        )?;
    }
    if !body_terminated {
        builder.ins().jump(continue_block, &[]);
    }

    builder.switch_to_block(continue_block);
    let one = builder.ins().iconst(ir::types::I64, 1);
    let current_idx = builder.use_var(idx_var);
    let next_idx = builder.ins().iadd(current_idx, one);
    builder.def_var(idx_var, next_idx);
    builder.ins().jump(header_block, &[]);

    builder.seal_block(body_block);
    builder.seal_block(continue_block);
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
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
                        type_defs,
                        enum_defs,
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
                        type_defs,
                        enum_defs,
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
                        type_defs,
                        enum_defs,
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
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
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
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
                    type_defs,
                    enum_defs,
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
                let (first_key_expr, first_val_expr) = &entries[0];
                let use_str_keys = is_known_string_expr_full(first_key_expr, owner, type_defs);
                let use_str_vals =
                    use_str_keys && is_known_string_expr_full(first_val_expr, owner, type_defs);
                let local_new =
                    if use_str_keys && use_str_vals {
                        let cap = builder.ins().iconst(ir::types::I64, entries.len() as i64);
                        let new_fn = module
                            .declare_func_in_func(runtime_fns.map_new_str_str_fn, builder.func);
                        let call = builder.ins().call(new_fn, &[cap]);
                        builder.inst_results(call).first().copied().ok_or_else(|| {
                            "map runtime call did not return map handle".to_string()
                        })?
                    } else if use_str_keys {
                        let new_fn = module
                            .declare_func_in_func(runtime_fns.map_new_str_i64_fn, builder.func);
                        let call = builder.ins().call(new_fn, &[]);
                        builder.inst_results(call).first().copied().ok_or_else(|| {
                            "map runtime call did not return map handle".to_string()
                        })?
                    } else {
                        let new_fn = module
                            .declare_func_in_func(runtime_fns.map_new_i64_i64_fn, builder.func);
                        let call = builder.ins().call(new_fn, &[]);
                        builder.inst_results(call).first().copied().ok_or_else(|| {
                            "map runtime call did not return map handle".to_string()
                        })?
                    };
                let map_handle = local_new;
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
                        type_defs,
                        enum_defs,
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
                        type_defs,
                        enum_defs,
                    )?;
                    if use_str_keys && use_str_vals {
                        let local_set = module.declare_func_in_func(
                            runtime_fns.container_set_str_str_fn,
                            builder.func,
                        );
                        builder.ins().call(local_set, &[map_handle, key, value]);
                    } else if use_str_keys {
                        if builder.func.dfg.value_type(value) != ir::types::I64 {
                            return Err(
                                "E3401: native map lowering currently supports Int values only for Map<Str, Int>"
                                    .to_string(),
                            );
                        }
                        let local_set = module.declare_func_in_func(
                            runtime_fns.container_set_str_i64_fn,
                            builder.func,
                        );
                        builder.ins().call(local_set, &[map_handle, key, value]);
                    } else {
                        if is_known_string_expr_full(key_expr, owner, type_defs) {
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
        MirExpr::Member {
            object,
            field,
            object_type: type_hint,
        } => {
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
                type_defs,
                enum_defs,
            )?;
            if field == "len" {
                let use_str_len = is_known_string_expr_full(object, owner, type_defs)
                    || matches!(&**object, MirExpr::Var(name)
                        if var_has_add_assignment(owner, name)
                        || var_has_str_call_assignment(owner, name, function_returns));
                let local_len = if use_str_len {
                    module.declare_func_in_func(runtime_fns.str_len_bytes_fn, builder.func)
                } else {
                    module.declare_func_in_func(runtime_fns.container_len_fn, builder.func)
                };
                let call = builder.ins().call(local_len, &[container]);
                return Ok(builder
                    .inst_results(call)
                    .first()
                    .copied()
                    .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0)));
            }
            if let Some(type_name) = type_hint {
                if let Some(fields) = type_defs.get(type_name) {
                    let slot_index =
                        fields.iter().position(|(f, _)| f == field).ok_or_else(|| {
                            format!("E3402: unknown field `{field}` on type `{type_name}`")
                        })?;
                    let (_, field_ty) = &fields[slot_index];
                    let offset_bytes = (slot_index as i64 * 8) as i32;
                    let load_ty = match field_ty.as_str() {
                        "Str" => ptr_ty,
                        "Bool" => ir::types::I8,
                        _ => ir::types::I64,
                    };
                    let loaded = builder.ins().load(
                        load_ty,
                        MemFlags::new(),
                        container,
                        Offset32::new(offset_bytes),
                    );
                    return Ok(loaded);
                }
            }
            return Err(format!(
                "E3402: member access `{field}` native lowering is not available in v0.1 backend"
            ));
        }
        MirExpr::Index {
            object,
            index,
            object_is_str,
        } => {
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
                type_defs,
                enum_defs,
            )?;
            let index_value = emit_expr(
                index,
                module,
                builder,
                locals,
                function_ids,
                function_returns,
                runtime_fns,
                ptr_ty,
                str_data_counter,
                owner,
                type_defs,
                enum_defs,
            )?;
            if *object_is_str {
                let local_get_byte =
                    module.declare_func_in_func(runtime_fns.str_get_byte_fn, builder.func);
                let call = builder
                    .ins()
                    .call(local_get_byte, &[object_value, index_value]);
                call_result_or_zero(builder, call)
            } else {
                // Inline list access: struct { tag:i64, len:i64, cap:i64, items:*i64 }
                let len = builder.ins().load(
                    ir::types::I64,
                    MemFlags::trusted(),
                    object_value,
                    Offset32::new(8),
                );
                let oob_block = builder.create_block();
                let ok_block = builder.create_block();

                let in_bounds = builder
                    .ins()
                    .icmp(IntCC::UnsignedLessThan, index_value, len);
                builder.ins().brif(in_bounds, ok_block, &[], oob_block, &[]);

                builder.switch_to_block(oob_block);
                builder.seal_block(oob_block);
                let local_panic = module.declare_func_in_func(runtime_fns.panic_fn, builder.func);
                let oob_msg = emit_string_data(
                    module,
                    builder,
                    "list index out of bounds",
                    ptr_ty,
                    str_data_counter,
                    owner,
                )?;
                builder.ins().call(local_panic, &[oob_msg]);
                builder
                    .ins()
                    .trap(cranelift_codegen::ir::TrapCode::unwrap_user(1));

                builder.switch_to_block(ok_block);
                builder.seal_block(ok_block);
                let items_ptr = builder.ins().load(
                    ptr_ty,
                    MemFlags::trusted(),
                    object_value,
                    Offset32::new(24),
                );
                let byte_offset = builder.ins().imul_imm(index_value, 8);
                let elem_addr = builder.ins().iadd(items_ptr, byte_offset);
                builder.ins().load(
                    ir::types::I64,
                    MemFlags::trusted(),
                    elem_addr,
                    Offset32::new(0),
                )
            }
        }
        MirExpr::Slice {
            object,
            start,
            end,
            object_is_str,
        } => {
            if !*object_is_str {
                return Err(
                    "E3410: native slicing currently supports Str operands only".to_string()
                );
            }
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
                type_defs,
                enum_defs,
            )?;
            let start_value = if let Some(start) = start {
                emit_expr(
                    start,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                    type_defs,
                    enum_defs,
                )?
            } else {
                builder.ins().iconst(ir::types::I64, 0)
            };
            let end_value = if let Some(end) = end {
                emit_expr(
                    end,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                    type_defs,
                    enum_defs,
                )?
            } else {
                let local_len =
                    module.declare_func_in_func(runtime_fns.str_len_bytes_fn, builder.func);
                let call = builder.ins().call(local_len, &[object_value]);
                call_result_or_zero(builder, call)
            };
            let local_slice = module.declare_func_in_func(runtime_fns.str_slice_fn, builder.func);
            let call = builder
                .ins()
                .call(local_slice, &[object_value, start_value, end_value]);
            call_result_or_zero(builder, call)
        }
        MirExpr::Call { callee, args } => {
            if let MirExpr::Var(name) = &**callee {
                if name == "__assign" {
                    if args.len() != 2 {
                        return Err("__assign expects two arguments".to_string());
                    }
                    if let MirExpr::Member {
                        object,
                        field,
                        object_type: Some(type_name),
                    } = &args[0]
                    {
                        if let Some(fields) = type_defs.get(type_name) {
                            let slot_index = fields
                                .iter()
                                .position(|(f, _)| f == field)
                                .ok_or_else(|| format!("E3402: unknown field `{field}`"))?;
                            let ptr = emit_expr(
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
                                type_defs,
                                enum_defs,
                            )?;
                            let value = emit_expr(
                                &args[1],
                                module,
                                builder,
                                locals,
                                function_ids,
                                function_returns,
                                runtime_fns,
                                ptr_ty,
                                str_data_counter,
                                owner,
                                type_defs,
                                enum_defs,
                            )?;
                            let offset_bytes = (slot_index as i64 * 8) as i32;
                            let store_ty = builder.func.dfg.value_type(value);
                            let to_store = if store_ty == ir::types::I8 {
                                builder.ins().sextend(ir::types::I64, value)
                            } else {
                                value
                            };
                            builder.ins().store(
                                MemFlags::new(),
                                to_store,
                                ptr,
                                Offset32::new(offset_bytes),
                            );
                            return Ok(builder.ins().iconst(ir::types::I64, 0));
                        }
                    }
                    return Err("E3499: __assign requires record field Member target".to_string());
                }
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
                        type_defs,
                        enum_defs,
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
                        type_defs,
                        enum_defs,
                    )?;
                    let local_new =
                        module.declare_func_in_func(runtime_fns.chan_new_fn, builder.func);
                    let call = builder.ins().call(local_new, &[capacity]);
                    let chan = builder.inst_results(call).first().copied().ok_or_else(|| {
                        "chan runtime call did not return channel handle".to_string()
                    })?;
                    return Ok(chan);
                }
                if name == "cpu_count" {
                    if !args.is_empty() {
                        return Err("`cpu_count` expects no arguments".to_string());
                    }
                    return Ok(builder.ins().iconst(ir::types::I64, 1));
                }
                if name == "len" {
                    if args.len() != 1 {
                        return Err("`len` expects one argument".to_string());
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
                        type_defs,
                        enum_defs,
                    )?;
                    let use_str_len = is_known_string_expr_full(&args[0], owner, type_defs);
                    let local_len = if use_str_len {
                        module.declare_func_in_func(runtime_fns.str_len_bytes_fn, builder.func)
                    } else {
                        module.declare_func_in_func(runtime_fns.container_len_fn, builder.func)
                    };
                    let call = builder.ins().call(local_len, &[arg0]);
                    return Ok(call_result_or_zero(builder, call));
                }
                if name == "min" || name == "max" {
                    if args.len() != 2 {
                        return Err(format!("`{name}` expects two arguments"));
                    }
                    let left = emit_expr(
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
                        type_defs,
                        enum_defs,
                    )?;
                    let right = emit_expr(
                        &args[1],
                        module,
                        builder,
                        locals,
                        function_ids,
                        function_returns,
                        runtime_fns,
                        ptr_ty,
                        str_data_counter,
                        owner,
                        type_defs,
                        enum_defs,
                    )?;
                    let left_ty = builder.func.dfg.value_type(left);
                    let right_ty = builder.func.dfg.value_type(right);
                    if left_ty == ir::types::F64 || right_ty == ir::types::F64 {
                        let left_f = coerce_numeric_to_f64(builder, left);
                        let right_f = coerce_numeric_to_f64(builder, right);
                        let cc = if name == "min" {
                            FloatCC::LessThan
                        } else {
                            FloatCC::GreaterThan
                        };
                        let cond = builder.ins().fcmp(cc, left_f, right_f);
                        return Ok(builder.ins().select(cond, left_f, right_f));
                    }
                    let cc = if name == "min" {
                        IntCC::SignedLessThan
                    } else {
                        IntCC::SignedGreaterThan
                    };
                    let cond = builder.ins().icmp(cc, left, right);
                    return Ok(builder.ins().select(cond, left, right));
                }
                if name == "sorted_desc" {
                    if args.len() != 1 {
                        return Err("`sorted_desc` expects one argument".to_string());
                    }
                    let input = emit_expr(
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
                        type_defs,
                        enum_defs,
                    )?;
                    let local_sort = module
                        .declare_func_in_func(runtime_fns.list_sort_desc_i64_fn, builder.func);
                    let sorted_call = builder.ins().call(local_sort, &[input]);
                    let sorted_handle = call_result_or_zero(builder, sorted_call);
                    let local_eq =
                        module.declare_func_in_func(runtime_fns.container_eq_fn, builder.func);
                    let eq_call = builder.ins().call(local_eq, &[input, sorted_handle]);
                    return Ok(call_result_or_zero(builder, eq_call));
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
                            type_defs,
                            enum_defs,
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
            if let MirExpr::Member { object, field, .. } = &**callee {
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
                        type_defs,
                        enum_defs,
                    )?);
                }
                if let Some((namespace, stdlib_field)) =
                    extract_stdlib_call_target_mir(callee.as_ref(), function_ids)
                {
                    if let Some(value) = emit_stdlib_namespace_call(
                        &namespace,
                        &stdlib_field,
                        &lowered_args,
                        module,
                        builder,
                        runtime_fns,
                        function_ids,
                        ptr_ty,
                        str_data_counter,
                        owner,
                        type_defs,
                    )? {
                        return Ok(value);
                    }
                }
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
                    type_defs,
                    enum_defs,
                )?;
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
                    "sort_desc" => {
                        if !lowered_args.is_empty() {
                            return Err("`.sort_desc()` expects no arguments".to_string());
                        }
                        let local_sort = module
                            .declare_func_in_func(runtime_fns.list_sort_desc_i64_fn, builder.func);
                        let call = builder.ins().call(local_sort, &[object_value]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ptr_ty, 0)));
                    }
                    "take" => {
                        if lowered_args.len() != 1 {
                            return Err("`.take()` expects one count argument".to_string());
                        }
                        let local_take =
                            module.declare_func_in_func(runtime_fns.list_take_i64_fn, builder.func);
                        let call = builder
                            .ins()
                            .call(local_take, &[object_value, lowered_args[0]]);
                        return Ok(builder
                            .inst_results(call)
                            .first()
                            .copied()
                            .unwrap_or_else(|| builder.ins().iconst(ptr_ty, 0)));
                    }
                    "len" => {
                        if !lowered_args.is_empty() {
                            return Err("`.len()` expects no arguments".to_string());
                        }
                        let use_str_len = is_known_string_expr_full(object, owner, type_defs)
                            || matches!(&**object, MirExpr::Var(name)
                                if var_has_add_assignment(owner, name)
                                || var_has_str_call_assignment(owner, name, function_returns));
                        let local_len = if use_str_len {
                            module.declare_func_in_func(runtime_fns.str_len_bytes_fn, builder.func)
                        } else {
                            module.declare_func_in_func(runtime_fns.container_len_fn, builder.func)
                        };
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
                        let local_get = module.declare_func_in_func(
                            runtime_fns.container_get_auto_i64_fn,
                            builder.func,
                        );
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

                        let tag = builder.ins().load(
                            ir::types::I64,
                            MemFlags::trusted(),
                            object_value,
                            Offset32::new(0),
                        );
                        let is_list = builder.ins().icmp_imm(IntCC::Equal, tag, 1);

                        let list_block = builder.create_block();
                        let map_block = builder.create_block();
                        let merge_block = builder.create_block();

                        builder.ins().brif(is_list, list_block, &[], map_block, &[]);

                        builder.switch_to_block(list_block);
                        builder.seal_block(list_block);
                        {
                            let len = builder.ins().load(
                                ir::types::I64,
                                MemFlags::trusted(),
                                object_value,
                                Offset32::new(8),
                            );
                            let oob_block = builder.create_block();
                            let ok_block = builder.create_block();
                            let in_bounds = builder.ins().icmp(IntCC::UnsignedLessThan, key, len);
                            builder.ins().brif(in_bounds, ok_block, &[], oob_block, &[]);

                            builder.switch_to_block(oob_block);
                            builder.seal_block(oob_block);
                            let local_panic =
                                module.declare_func_in_func(runtime_fns.panic_fn, builder.func);
                            let oob_msg = emit_string_data(
                                module,
                                builder,
                                "list.set index out of bounds",
                                ptr_ty,
                                str_data_counter,
                                owner,
                            )?;
                            builder.ins().call(local_panic, &[oob_msg]);
                            builder
                                .ins()
                                .trap(cranelift_codegen::ir::TrapCode::unwrap_user(1));

                            builder.switch_to_block(ok_block);
                            builder.seal_block(ok_block);
                            let items_ptr = builder.ins().load(
                                ptr_ty,
                                MemFlags::trusted(),
                                object_value,
                                Offset32::new(24),
                            );
                            let byte_offset = builder.ins().imul_imm(key, 8);
                            let elem_addr = builder.ins().iadd(items_ptr, byte_offset);
                            builder.ins().store(
                                MemFlags::trusted(),
                                value,
                                elem_addr,
                                Offset32::new(0),
                            );
                            builder.ins().jump(merge_block, &[]);
                        }

                        builder.switch_to_block(map_block);
                        builder.seal_block(map_block);
                        {
                            let local_set = module.declare_func_in_func(
                                runtime_fns.container_set_auto_i64_fn,
                                builder.func,
                            );
                            builder.ins().call(local_set, &[object_value, key, value]);
                            builder.ins().jump(merge_block, &[]);
                        }

                        builder.switch_to_block(merge_block);
                        builder.seal_block(merge_block);
                        return Ok(builder.ins().iconst(ir::types::I64, 0));
                    }
                    "contains" => {
                        if lowered_args.len() != 1 {
                            return Err("`.contains()` expects one key argument".to_string());
                        }
                        let key = lowered_args[0];
                        let local_contains = module.declare_func_in_func(
                            runtime_fns.container_contains_auto_i64_fn,
                            builder.func,
                        );
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
                        let local_remove = module.declare_func_in_func(
                            runtime_fns.container_remove_auto_i64_fn,
                            builder.func,
                        );
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
        MirExpr::Binary { left, op, right } if op == "And" || op == "Or" => {
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
                type_defs,
                enum_defs,
            )?;
            let l_ty = builder.func.dfg.value_type(l);
            let zero = builder.ins().iconst(l_ty, 0);
            let l_bool = builder.ins().icmp(IntCC::NotEqual, l, zero);

            let rhs_block = builder.create_block();
            let merge_block = builder.create_block();

            let slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                8,
                8,
            ));

            if op == "And" {
                let false_val = builder.ins().iconst(ir::types::I64, 0);
                builder.ins().stack_store(false_val, slot, Offset32::new(0));
                builder.ins().brif(l_bool, rhs_block, &[], merge_block, &[]);
            } else {
                let true_val = builder.ins().iconst(ir::types::I64, 1);
                builder.ins().stack_store(true_val, slot, Offset32::new(0));
                builder.ins().brif(l_bool, merge_block, &[], rhs_block, &[]);
            }

            builder.switch_to_block(rhs_block);
            builder.seal_block(rhs_block);
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
                type_defs,
                enum_defs,
            )?;
            let r_ty = builder.func.dfg.value_type(r);
            let r_zero = builder.ins().iconst(r_ty, 0);
            let r_bool = builder.ins().icmp(IntCC::NotEqual, r, r_zero);
            let r_result = builder.ins().uextend(ir::types::I64, r_bool);
            builder.ins().stack_store(r_result, slot, Offset32::new(0));
            builder.ins().jump(merge_block, &[]);

            builder.switch_to_block(merge_block);
            builder.seal_block(merge_block);
            builder
                .ins()
                .stack_load(ir::types::I64, slot, Offset32::new(0))
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
                type_defs,
                enum_defs,
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
                type_defs,
                enum_defs,
            )?;
            let l_ty = builder.func.dfg.value_type(l);
            let r_ty = builder.func.dfg.value_type(r);
            let numeric_is_float = l_ty == ir::types::F64 || r_ty == ir::types::F64;
            let l_num = if numeric_is_float {
                coerce_numeric_to_f64(builder, l)
            } else {
                l
            };
            let r_num = if numeric_is_float {
                coerce_numeric_to_f64(builder, r)
            } else {
                r
            };
            match op.as_str() {
                "Add" => {
                    if is_known_string_expr_full(left, owner, type_defs)
                        || is_known_string_expr_full(right, owner, type_defs)
                    {
                        let local_concat =
                            module.declare_func_in_func(runtime_fns.str_concat_fn, builder.func);
                        let call = builder.ins().call(local_concat, &[l, r]);
                        builder.inst_results(call).first().copied().ok_or_else(|| {
                            "string concat runtime call returned no value".to_string()
                        })?
                    } else if numeric_is_float {
                        builder.ins().fadd(l_num, r_num)
                    } else {
                        builder.ins().iadd(l, r)
                    }
                }
                "Sub" => {
                    if numeric_is_float {
                        builder.ins().fsub(l_num, r_num)
                    } else {
                        builder.ins().isub(l, r)
                    }
                }
                "Mul" => {
                    if numeric_is_float {
                        builder.ins().fmul(l_num, r_num)
                    } else {
                        builder.ins().imul(l, r)
                    }
                }
                "Div" => {
                    if numeric_is_float {
                        builder.ins().fdiv(l_num, r_num)
                    } else {
                        builder.ins().sdiv(l, r)
                    }
                }
                "Eq" | "Ne" => {
                    let is_ne = op == "Ne";
                    if is_known_string_expr_full(left, owner, type_defs)
                        || is_known_string_expr_full(right, owner, type_defs)
                    {
                        let local_eq =
                            module.declare_func_in_func(runtime_fns.str_eq_fn, builder.func);
                        let call = builder.ins().call(local_eq, &[l, r]);
                        let eq_value = call_result_or_zero(builder, call);
                        if is_ne {
                            let zero = builder.ins().iconst(ir::types::I64, 0);
                            let cmp = builder.ins().icmp(IntCC::Equal, eq_value, zero);
                            builder.ins().uextend(ir::types::I64, cmp)
                        } else {
                            eq_value
                        }
                    } else if is_known_container_expr(left) && is_known_container_expr(right) {
                        let local_eq =
                            module.declare_func_in_func(runtime_fns.container_eq_fn, builder.func);
                        let call = builder.ins().call(local_eq, &[l, r]);
                        let eq_value = call_result_or_zero(builder, call);
                        if is_ne {
                            let zero = builder.ins().iconst(ir::types::I64, 0);
                            let cmp = builder.ins().icmp(IntCC::Equal, eq_value, zero);
                            builder.ins().uextend(ir::types::I64, cmp)
                        } else {
                            eq_value
                        }
                    } else if numeric_is_float {
                        let cc = if is_ne {
                            FloatCC::NotEqual
                        } else {
                            FloatCC::Equal
                        };
                        let cmp = builder.ins().fcmp(cc, l_num, r_num);
                        builder.ins().uextend(ir::types::I64, cmp)
                    } else {
                        let cc = if is_ne { IntCC::NotEqual } else { IntCC::Equal };
                        let cmp = builder.ins().icmp(cc, l, r);
                        builder.ins().uextend(ir::types::I64, cmp)
                    }
                }
                "Lt" | "Le" | "Gt" | "Ge" => {
                    if numeric_is_float {
                        let cc = match op.as_str() {
                            "Lt" => FloatCC::LessThan,
                            "Le" => FloatCC::LessThanOrEqual,
                            "Gt" => FloatCC::GreaterThan,
                            "Ge" => FloatCC::GreaterThanOrEqual,
                            _ => FloatCC::Equal,
                        };
                        let cmp = builder.ins().fcmp(cc, l_num, r_num);
                        builder.ins().uextend(ir::types::I64, cmp)
                    } else {
                        let cc = match op.as_str() {
                            "Lt" => IntCC::SignedLessThan,
                            "Le" => IntCC::SignedLessThanOrEqual,
                            "Gt" => IntCC::SignedGreaterThan,
                            "Ge" => IntCC::SignedGreaterThanOrEqual,
                            _ => IntCC::Equal,
                        };
                        let cmp = builder.ins().icmp(cc, l, r);
                        builder.ins().uextend(ir::types::I64, cmp)
                    }
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
                type_defs,
                enum_defs,
            )?;
            match op.as_str() {
                "Neg" => {
                    if builder.func.dfg.value_type(v) == ir::types::F64 {
                        builder.ins().fneg(v)
                    } else {
                        builder.ins().ineg(v)
                    }
                }
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
        MirExpr::Async { expr } => {
            let inner = emit_expr(
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
                type_defs,
                enum_defs,
            )?;
            let inner_ty = builder.func.dfg.value_type(inner);
            if inner_ty == ptr_ty {
                let local = module.declare_func_in_func(runtime_fns.async_ptr_fn, builder.func);
                let call = builder.ins().call(local, &[inner]);
                call_result_or_zero(builder, call)
            } else if inner_ty == ir::types::I64 {
                let local = module.declare_func_in_func(runtime_fns.async_i64_fn, builder.func);
                let call = builder.ins().call(local, &[inner]);
                call_result_or_zero(builder, call)
            } else {
                inner
            }
        }
        MirExpr::Await { expr } => {
            let inner = emit_expr(
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
                type_defs,
                enum_defs,
            )?;
            let inner_ty = builder.func.dfg.value_type(inner);
            if inner_ty == ptr_ty {
                let local = module.declare_func_in_func(runtime_fns.await_ptr_fn, builder.func);
                let call = builder.ins().call(local, &[inner]);
                call_result_or_zero(builder, call)
            } else if inner_ty == ir::types::I64 {
                let local = module.declare_func_in_func(runtime_fns.await_i64_fn, builder.func);
                let call = builder.ins().call(local, &[inner]);
                call_result_or_zero(builder, call)
            } else {
                inner
            }
        }
        MirExpr::Old { expr } => emit_expr(
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
            type_defs,
            enum_defs,
        )?,
        MirExpr::Question { expr } => {
            let result_ptr = emit_expr(
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
                type_defs,
                enum_defs,
            )?;
            let tag = builder.ins().load(
                ir::types::I64,
                MemFlags::new(),
                result_ptr,
                Offset32::new(0),
            );
            let one = builder.ins().iconst(ir::types::I64, 1);
            let is_err = builder.ins().icmp(IntCC::Equal, tag, one);
            let err_block = builder.create_block();
            let ok_block = builder.create_block();
            builder.ins().brif(is_err, err_block, &[], ok_block, &[]);

            builder.switch_to_block(err_block);
            builder.ins().return_(&[result_ptr]);
            builder.seal_block(err_block);

            builder.switch_to_block(ok_block);
            builder.seal_block(ok_block);
            builder.ins().load(
                ir::types::I64,
                MemFlags::new(),
                result_ptr,
                Offset32::new(8),
            )
        }
        MirExpr::ResultOk { expr } => {
            let inner = emit_expr(
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
                type_defs,
                enum_defs,
            )?;
            let local_alloc =
                module.declare_func_in_func(runtime_fns.record_alloc_fn, builder.func);
            let slot_count = builder.ins().iconst(ir::types::I64, 2);
            let alloc_call = builder.ins().call(local_alloc, &[slot_count]);
            let ptr = builder
                .inst_results(alloc_call)
                .first()
                .copied()
                .ok_or_else(|| "record_alloc did not return".to_string())?;
            let tag = builder.ins().iconst(ir::types::I64, 0);
            builder
                .ins()
                .store(MemFlags::new(), tag, ptr, Offset32::new(0));
            let inner_ty = builder.func.dfg.value_type(inner);
            let to_store = if inner_ty == ir::types::I8 {
                builder.ins().sextend(ir::types::I64, inner)
            } else {
                inner
            };
            builder
                .ins()
                .store(MemFlags::new(), to_store, ptr, Offset32::new(8));
            ptr
        }
        MirExpr::ResultErr { expr } => {
            let inner = emit_expr(
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
                type_defs,
                enum_defs,
            )?;
            let local_alloc =
                module.declare_func_in_func(runtime_fns.record_alloc_fn, builder.func);
            let slot_count = builder.ins().iconst(ir::types::I64, 2);
            let alloc_call = builder.ins().call(local_alloc, &[slot_count]);
            let ptr = builder
                .inst_results(alloc_call)
                .first()
                .copied()
                .ok_or_else(|| "record_alloc did not return".to_string())?;
            let tag = builder.ins().iconst(ir::types::I64, 1);
            builder
                .ins()
                .store(MemFlags::new(), tag, ptr, Offset32::new(0));
            let inner_ty = builder.func.dfg.value_type(inner);
            let to_store = if inner_ty == ir::types::I8 {
                builder.ins().sextend(ir::types::I64, inner)
            } else {
                inner
            };
            builder
                .ins()
                .store(MemFlags::new(), to_store, ptr, Offset32::new(8));
            ptr
        }
        MirExpr::DotResult => builder.ins().iconst(ir::types::I64, 0),
        MirExpr::Constructor { type_name, fields } => {
            let field_list = type_defs
                .get(type_name)
                .ok_or_else(|| format!("E3499: unknown type `{type_name}` in constructor"))?;
            let slot_count = field_list.len() as i64;
            let local_alloc =
                module.declare_func_in_func(runtime_fns.record_alloc_fn, builder.func);
            let slot_const = builder.ins().iconst(ir::types::I64, slot_count);
            let alloc_call = builder.ins().call(local_alloc, &[slot_const]);
            let ptr = builder
                .inst_results(alloc_call)
                .first()
                .copied()
                .ok_or_else(|| "record_alloc did not return".to_string())?;
            let field_order: Vec<&str> = field_list.iter().map(|(f, _)| f.as_str()).collect();
            for (fname, fval) in fields {
                let slot_index = field_order
                    .iter()
                    .position(|x| *x == fname)
                    .ok_or_else(|| format!("E3499: unknown field `{fname}` in constructor"))?;
                let value = emit_expr(
                    fval,
                    module,
                    builder,
                    locals,
                    function_ids,
                    function_returns,
                    runtime_fns,
                    ptr_ty,
                    str_data_counter,
                    owner,
                    type_defs,
                    enum_defs,
                )?;
                let (_, _field_ty) = &field_list[slot_index];
                let offset_bytes = (slot_index as i64 * 8) as i32;
                let store_ty = builder.func.dfg.value_type(value);
                let to_store = if store_ty == ir::types::I8 {
                    builder.ins().sextend(ir::types::I64, value)
                } else {
                    value
                };
                builder
                    .ins()
                    .store(MemFlags::new(), to_store, ptr, Offset32::new(offset_bytes));
            }
            ptr
        }
        MirExpr::EnumVariant { enum_name, variant } => {
            let variants = enum_defs
                .get(enum_name)
                .ok_or_else(|| format!("E3499: unknown enum `{enum_name}`"))?;
            let tag = variants
                .iter()
                .position(|v| v == variant)
                .ok_or_else(|| format!("E3499: unknown variant `{enum_name}.{variant}`"))?
                as i64;
            builder.ins().iconst(ir::types::I64, tag)
        }
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

fn call_result_or_zero(builder: &mut FunctionBuilder<'_>, call: ir::Inst) -> ir::Value {
    builder
        .inst_results(call)
        .first()
        .copied()
        .unwrap_or_else(|| builder.ins().iconst(ir::types::I64, 0))
}

fn coerce_numeric_to_f64(builder: &mut FunctionBuilder<'_>, value: ir::Value) -> ir::Value {
    let ty = builder.func.dfg.value_type(value);
    if ty == ir::types::F64 {
        return value;
    }
    if ty == ir::types::I64 {
        return builder.ins().fcvt_from_sint(ir::types::F64, value);
    }
    if ty == ir::types::I8 {
        let widened = builder.ins().sextend(ir::types::I64, value);
        return builder.ins().fcvt_from_sint(ir::types::F64, widened);
    }
    value
}

fn json_codec_schema(
    type_name: &str,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
) -> Option<String> {
    json_codec_schema_recursive(type_name, type_defs, 0)
}

fn json_codec_schema_recursive(
    type_name: &str,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
    depth: usize,
) -> Option<String> {
    if depth > 16 {
        eprintln!(
            "warning: json codec schema for `{type_name}` exceeds maximum nesting depth (16); \
             deeply nested fields will be serialized as empty objects"
        );
        return None;
    }
    let fields = type_defs.get(type_name)?;
    let mut schema = String::new();
    for (idx, (field_name, field_ty)) in fields.iter().enumerate() {
        if idx > 0 {
            schema.push(';');
        }
        schema.push_str(field_name);
        schema.push(':');
        if type_defs.contains_key(field_ty.as_str()) {
            if let Some(nested) = json_codec_schema_recursive(field_ty, type_defs, depth + 1) {
                schema.push('{');
                schema.push_str(&nested);
                schema.push('}');
            } else {
                schema.push_str("Str");
            }
        } else {
            schema.push_str(field_ty);
        }
    }
    Some(schema)
}

fn collect_member_chain_mir(expr: &MirExpr, parts: &mut Vec<String>) -> bool {
    match expr {
        MirExpr::Var(name) => {
            parts.push(name.clone());
            true
        }
        MirExpr::Member { object, field, .. } => {
            if collect_member_chain_mir(object, parts) {
                parts.push(field.clone());
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn extract_stdlib_call_target_mir_static(callee: &MirExpr) -> Option<(String, String)> {
    extract_stdlib_call_target_mir(callee, &BTreeMap::new())
}

fn extract_stdlib_call_target_mir(
    callee: &MirExpr,
    function_ids: &BTreeMap<String, FuncId>,
) -> Option<(String, String)> {
    let MirExpr::Member { .. } = callee else {
        return None;
    };
    let mut parts = Vec::new();
    if !collect_member_chain_mir(callee, &mut parts) || parts.len() < 2 {
        return None;
    }
    let root = parts.first()?;
    let is_dynamic = function_ids
        .keys()
        .any(|k| k.starts_with("__ns_call__") && k[11..].starts_with(root.as_str()));
    if !is_dynamic
        && !matches!(
            root.as_str(),
            "simd"
                | "json"
                | "bench"
                | "time"
                | "path"
                | "fs"
                | "net"
                | "convert"
                | "math"
                | "str_builder"
                | "text"
                | "encoding"
                | "log"
                | "env"
                | "cli"
                | "regex"
                | "http"
        )
    {
        return None;
    }
    let field = parts.pop()?;
    let namespace = parts.join(".");
    Some((namespace, field))
}

#[allow(clippy::too_many_arguments)]
fn emit_stdlib_namespace_call(
    namespace: &str,
    field: &str,
    lowered_args: &[ir::Value],
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder<'_>,
    runtime_fns: RuntimeFunctions,
    function_ids: &BTreeMap<String, FuncId>,
    ptr_ty: ir::Type,
    str_data_counter: &mut usize,
    owner: &MirFunction,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
) -> Result<Option<ir::Value>, String> {
    let ns_key = format!("__ns_call__{namespace}.{field}");
    if let Some(&func_id) = function_ids.get(&ns_key) {
        let local = module.declare_func_in_func(func_id, builder.func);
        let call = builder.ins().call(local, lowered_args);
        return Ok(Some(call_result_or_zero(builder, call)));
    }

    let call_one_arg = |func: FuncId,
                        arg0: ir::Value,
                        module: &mut ObjectModule,
                        builder: &mut FunctionBuilder<'_>|
     -> ir::Value {
        let local = module.declare_func_in_func(func, builder.func);
        let call = builder.ins().call(local, &[arg0]);
        call_result_or_zero(builder, call)
    };
    let call_two_args = |func: FuncId,
                         arg0: ir::Value,
                         arg1: ir::Value,
                         module: &mut ObjectModule,
                         builder: &mut FunctionBuilder<'_>|
     -> ir::Value {
        let local = module.declare_func_in_func(func, builder.func);
        let call = builder.ins().call(local, &[arg0, arg1]);
        call_result_or_zero(builder, call)
    };

    let expect_arity = |expected: usize| -> Result<(), String> {
        if lowered_args.len() != expected {
            return Err(format!(
                "E3411: `{namespace}.{field}` expects {expected} argument(s), got {}",
                lowered_args.len()
            ));
        }
        Ok(())
    };

    if namespace == "json" {
        if let Some(type_name) = field.strip_prefix("encode_") {
            expect_arity(1)?;
            let schema = json_codec_schema(type_name, type_defs).ok_or_else(|| {
                format!("E3411: unknown json codec target type `{type_name}` for `{field}`")
            })?;
            let schema_ptr =
                emit_string_data(module, builder, &schema, ptr_ty, str_data_counter, owner)?;
            let local =
                module.declare_func_in_func(runtime_fns.json_encode_record_fn, builder.func);
            let call = builder.ins().call(local, &[lowered_args[0], schema_ptr]);
            return Ok(Some(call_result_or_zero(builder, call)));
        }
        if let Some(type_name) = field.strip_prefix("decode_") {
            expect_arity(2)?;
            let fields = type_defs.get(type_name).ok_or_else(|| {
                format!("E3411: unknown json codec target type `{type_name}` for `{field}`")
            })?;
            let schema = json_codec_schema(type_name, type_defs)
                .ok_or_else(|| format!("E3411: missing json schema for type `{type_name}`"))?;
            let schema_ptr =
                emit_string_data(module, builder, &schema, ptr_ty, str_data_counter, owner)?;
            let local_alloc =
                module.declare_func_in_func(runtime_fns.record_alloc_fn, builder.func);
            let slot_const = builder.ins().iconst(ir::types::I64, fields.len() as i64);
            let alloc_call = builder.ins().call(local_alloc, &[slot_const]);
            let out_record = call_result_or_zero(builder, alloc_call);
            let local_decode =
                module.declare_func_in_func(runtime_fns.json_decode_record_fn, builder.func);
            let call = builder.ins().call(
                local_decode,
                &[lowered_args[0], schema_ptr, lowered_args[1], out_record],
            );
            return Ok(Some(call_result_or_zero(builder, call)));
        }
        if field == "encode" && lowered_args.len() == 1 {
            return Err(format!(
                "E3411: `json.encode` could not determine the struct type of its argument. \
                 Bind the argument to a variable first: `tmp := expr; json.encode(tmp)`"
            ));
        }
        if field == "decode" && lowered_args.len() == 2 {
            return Err(format!(
                "E3411: `json.decode` could not determine the struct type of its fallback argument. \
                 Bind the fallback to a variable first: `fb := expr; json.decode(raw, fb)`"
            ));
        }
    }

    let value = match (namespace, field) {
        ("simd", "f64x2_splat") => {
            expect_arity(1)?;
            let scalar = lowered_args[0];
            let vec = builder.ins().splat(ir::types::F64X2, scalar);
            let slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                16,
                16,
            ));
            builder.ins().stack_store(vec, slot, Offset32::new(0));
            builder.ins().stack_addr(ptr_ty, slot, Offset32::new(0))
        }
        ("simd", "f64x2_make") => {
            expect_arity(2)?;
            let vec = builder.ins().splat(ir::types::F64X2, lowered_args[0]);
            let vec = builder.ins().insertlane(vec, lowered_args[1], 1);
            let slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                16,
                16,
            ));
            builder.ins().stack_store(vec, slot, Offset32::new(0));
            builder.ins().stack_addr(ptr_ty, slot, Offset32::new(0))
        }
        ("simd", "f64x2_add") => {
            expect_arity(2)?;
            let a = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[0],
                Offset32::new(0),
            );
            let b = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[1],
                Offset32::new(0),
            );
            let result = builder.ins().fadd(a, b);
            let slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                16,
                16,
            ));
            builder.ins().stack_store(result, slot, Offset32::new(0));
            builder.ins().stack_addr(ptr_ty, slot, Offset32::new(0))
        }
        ("simd", "f64x2_sub") => {
            expect_arity(2)?;
            let a = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[0],
                Offset32::new(0),
            );
            let b = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[1],
                Offset32::new(0),
            );
            let result = builder.ins().fsub(a, b);
            let slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                16,
                16,
            ));
            builder.ins().stack_store(result, slot, Offset32::new(0));
            builder.ins().stack_addr(ptr_ty, slot, Offset32::new(0))
        }
        ("simd", "f64x2_mul") => {
            expect_arity(2)?;
            let a = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[0],
                Offset32::new(0),
            );
            let b = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[1],
                Offset32::new(0),
            );
            let result = builder.ins().fmul(a, b);
            let slot = builder.create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                16,
                16,
            ));
            builder.ins().stack_store(result, slot, Offset32::new(0));
            builder.ins().stack_addr(ptr_ty, slot, Offset32::new(0))
        }
        ("simd", "f64x2_gt") => {
            expect_arity(2)?;
            let a = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[0],
                Offset32::new(0),
            );
            let b = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[1],
                Offset32::new(0),
            );
            let cmp = builder
                .ins()
                .fcmp(ir::condcodes::FloatCC::GreaterThan, a, b);
            let lane0 = builder.ins().extractlane(cmp, 0);
            let lane1 = builder.ins().extractlane(cmp, 1);
            let l0_i64 = builder.ins().sextend(ir::types::I64, lane0);
            let l1_i64 = builder.ins().sextend(ir::types::I64, lane1);
            let shifted = builder.ins().ishl_imm(l1_i64, 1);
            builder.ins().bor(l0_i64, shifted)
        }
        ("simd", "f64x2_extract") => {
            expect_arity(2)?;
            let vec = builder.ins().load(
                ir::types::F64X2,
                MemFlags::trusted(),
                lowered_args[0],
                Offset32::new(0),
            );
            let lane_idx = lowered_args[1];
            let zero = builder.ins().iconst(ir::types::I64, 0);
            let is_zero = builder.ins().icmp(IntCC::Equal, lane_idx, zero);
            let lane0 = builder.ins().extractlane(vec, 0);
            let lane1 = builder.ins().extractlane(vec, 1);
            builder.ins().select(is_zero, lane0, lane1)
        }
        ("json", "from_map") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_from_str_str_map_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "new") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_new_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "begin_object") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_begin_object_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "end_object") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_end_object_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "begin_array") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_begin_array_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "end_array") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_end_array_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "key") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.json_builder_key_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        ("json.builder", "value_null") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_value_null_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        ("json.builder", "value_bool") => {
            expect_arity(2)?;
            // Bool lowers to I8; runtime ABI uses I64 for the flag.
            let flag = builder.ins().uextend(ir::types::I64, lowered_args[1]);
            call_two_args(
                runtime_fns.json_builder_value_bool_fn,
                lowered_args[0],
                flag,
                module,
                builder,
            )
        }
        ("json.builder", "value_i64") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.json_builder_value_i64_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        ("json.builder", "value_f64") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.json_builder_value_f64_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        ("json.builder", "value_str") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.json_builder_value_str_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        ("json.builder", "value_json") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.json_builder_value_json_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        ("json.builder", "finish") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.json_builder_finish_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "md5_hex") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_md5_hex_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "md5_bytes_hex") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_md5_bytes_hex_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "json_canonical") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_json_canonical_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "json_repeat_array") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.bench_json_repeat_array_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "http_server_bench") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_http_server_bench_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "secp256k1") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_secp256k1_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "edigits") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_edigits_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_listen") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.bench_net_listen_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_listener_port") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_net_listener_port_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_accept") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_net_accept_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_connect") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.bench_net_connect_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_read") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.bench_net_read_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_write") => {
            expect_arity(2)?;
            call_two_args(
                runtime_fns.bench_net_write_fn,
                lowered_args[0],
                lowered_args[1],
                module,
                builder,
            )
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_close") => {
            expect_arity(1)?;
            call_one_arg(
                runtime_fns.bench_net_close_fn,
                lowered_args[0],
                module,
                builder,
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(value))
}

fn default_value(builder: &mut FunctionBuilder<'_>, ty: &MirType, ptr_ty: ir::Type) -> ir::Value {
    match ty {
        MirType::I64 | MirType::Unknown => builder.ins().iconst(ir::types::I64, 0),
        MirType::F64 => builder.ins().f64const(0.0),
        MirType::Bool => builder.ins().iconst(ir::types::I8, 0),
        MirType::Str | MirType::Json | MirType::JsonBuilder | MirType::Result => {
            builder.ins().iconst(ptr_ty, 0)
        }
        MirType::Void => builder.ins().iconst(ir::types::I64, 0),
    }
}

fn is_known_string_expr(expr: &MirExpr) -> bool {
    match expr {
        MirExpr::Str(_) => true,
        MirExpr::Binary { left, op, right } if op == "Add" => {
            is_known_string_expr(left) || is_known_string_expr(right)
        }
        MirExpr::Call { callee, .. } => {
            if let Some((namespace, field)) = extract_stdlib_call_target_mir_static(callee.as_ref())
            {
                return match namespace.as_str() {
                    "path" => field == "join" || field == "parent" || field == "basename",
                    "fs" => field == "read_text",
                    "net" => field == "read" || field == "resolve",
                    "convert" => {
                        field == "to_str" || field == "to_str_f64" || field == "format_f64"
                    }
                    "text" => {
                        field == "trim"
                            || field == "replace"
                            || field == "to_lower"
                            || field == "to_upper"
                            || field == "split_part"
                    }
                    "encoding" => {
                        field == "hex_encode"
                            || field == "hex_decode"
                            || field == "base64_encode"
                            || field == "base64_decode"
                            || field == "url_encode"
                            || field == "url_decode"
                    }
                    "env" => field == "get" || field == "get_required",
                    "cli" => field == "arg",
                    "json" => field.starts_with("encode_"),
                    "json.builder" => field == "finish",
                    "regex" => field == "replace_all",
                    "http" => {
                        field == "status_text"
                            || field == "build_request_line"
                            || field == "build_response"
                            || field == "send"
                            || field == "response"
                            || field == "get"
                            || field == "post"
                            || field == "request"
                    }
                    "bench" => {
                        field == "md5_hex"
                            || field == "md5_bytes_hex"
                            || field == "json_canonical"
                            || field == "json_repeat_array"
                            || field == "secp256k1"
                            || field == "edigits"
                            || field == "net_read"
                    }
                    _ => false,
                };
            }
            false
        }
        MirExpr::Slice { object_is_str, .. } => *object_is_str,
        _ => false,
    }
}

fn collect_var_string_assignments(
    stmts: &[MirStmt],
    target: &str,
    saw_assignment: &mut bool,
    all_string: &mut bool,
) {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { name, expr } | MirStmt::Assign { name, expr } => {
                if name == target {
                    *saw_assignment = true;
                    if !is_known_string_expr(expr) {
                        *all_string = false;
                    }
                }
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_var_string_assignments(then_body, target, saw_assignment, all_string);
                collect_var_string_assignments(else_body, target, saw_assignment, all_string);
            }
            MirStmt::For { body, .. }
            | MirStmt::While { body, .. }
            | MirStmt::Repeat { body, .. } => {
                collect_var_string_assignments(body, target, saw_assignment, all_string);
            }
            _ => {}
        }
    }
}

fn var_has_add_assignment_in_stmts(stmts: &[MirStmt], target: &str) -> bool {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { name, expr } | MirStmt::Assign { name, expr } => {
                if name == target && matches!(expr, MirExpr::Binary { op, .. } if op == "Add") {
                    return true;
                }
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if var_has_add_assignment_in_stmts(then_body, target)
                    || var_has_add_assignment_in_stmts(else_body, target)
                {
                    return true;
                }
            }
            MirStmt::For { body, .. }
            | MirStmt::While { body, .. }
            | MirStmt::Repeat { body, .. } => {
                if var_has_add_assignment_in_stmts(body, target) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn var_has_add_assignment(owner: &MirFunction, name: &str) -> bool {
    var_has_add_assignment_in_stmts(&owner.body, name)
}

fn var_has_str_call_assignment_in_stmts(
    stmts: &[MirStmt],
    target: &str,
    function_returns: &BTreeMap<String, MirType>,
) -> bool {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { name, expr } | MirStmt::Assign { name, expr } => {
                if name == target {
                    if let MirExpr::Call { callee, .. } = expr {
                        if let MirExpr::Var(fn_name) = &**callee {
                            if matches!(function_returns.get(fn_name), Some(MirType::Str)) {
                                return true;
                            }
                        }
                    }
                }
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if var_has_str_call_assignment_in_stmts(then_body, target, function_returns)
                    || var_has_str_call_assignment_in_stmts(else_body, target, function_returns)
                {
                    return true;
                }
            }
            MirStmt::For { body, .. }
            | MirStmt::While { body, .. }
            | MirStmt::Repeat { body, .. } => {
                if var_has_str_call_assignment_in_stmts(body, target, function_returns) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn var_has_str_call_assignment(
    owner: &MirFunction,
    name: &str,
    function_returns: &BTreeMap<String, MirType>,
) -> bool {
    var_has_str_call_assignment_in_stmts(&owner.body, name, function_returns)
}

fn is_var_known_string_in_owner(owner: &MirFunction, name: &str) -> bool {
    for param in &owner.params {
        if param.name == name {
            return param.ty == MirType::Str;
        }
    }
    let mut saw_assignment = false;
    let mut all_string = true;
    collect_var_string_assignments(&owner.body, name, &mut saw_assignment, &mut all_string);
    saw_assignment && all_string
}

fn is_known_string_expr_with_owner(expr: &MirExpr, owner: &MirFunction) -> bool {
    is_known_string_expr_full(expr, owner, &BTreeMap::new())
}

fn is_known_string_expr_full(
    expr: &MirExpr,
    owner: &MirFunction,
    type_defs: &BTreeMap<String, Vec<(String, String)>>,
) -> bool {
    if is_known_string_expr(expr) {
        return true;
    }
    match expr {
        MirExpr::Var(name) => is_var_known_string_in_owner(owner, name),
        MirExpr::Binary { left, op, right } if op == "Add" => {
            is_known_string_expr_full(left, owner, type_defs)
                || is_known_string_expr_full(right, owner, type_defs)
        }
        MirExpr::Member {
            field, object_type, ..
        } => {
            if let Some(type_name) = object_type {
                if let Some(fields) = type_defs.get(type_name) {
                    return fields.iter().any(|(f, t)| f == field && t == "Str");
                }
            }
            false
        }
        _ => false,
    }
}

fn is_known_container_expr(expr: &MirExpr) -> bool {
    matches!(expr, MirExpr::List(_) | MirExpr::Map(_))
        || matches!(
            expr,
            MirExpr::Call { callee, .. }
                if matches!(&**callee, MirExpr::Member { field, .. } if field == "sort_desc" || field == "take")
        )
}

fn infer_mir_expr_type(
    expr: &MirExpr,
    owner: &MirFunction,
    function_returns: &BTreeMap<String, MirType>,
    locals_ty: &BTreeMap<String, MirType>,
) -> MirType {
    match expr {
        MirExpr::Float(_) => MirType::F64,
        MirExpr::Bool(_) => MirType::Bool,
        MirExpr::Str(_) => MirType::Str,
        MirExpr::Int(_) => MirType::I64,
        MirExpr::Var(name) => {
            if let Some(ty) = locals_ty.get(name) {
                return ty.clone();
            }
            for param in &owner.params {
                if param.name == *name {
                    return param.ty.clone();
                }
            }
            MirType::I64
        }
        MirExpr::Binary { left, op, right } if op == "Add" => {
            let lt = infer_mir_expr_type(left, owner, function_returns, locals_ty);
            let rt = infer_mir_expr_type(right, owner, function_returns, locals_ty);
            if lt == MirType::Str || rt == MirType::Str {
                MirType::Str
            } else if lt == MirType::F64 || rt == MirType::F64 {
                MirType::F64
            } else {
                MirType::I64
            }
        }
        MirExpr::Binary { left, op, right }
            if op == "Sub" || op == "Mul" || op == "Div" || op == "Mod" =>
        {
            let lt = infer_mir_expr_type(left, owner, function_returns, locals_ty);
            let rt = infer_mir_expr_type(right, owner, function_returns, locals_ty);
            if lt == MirType::F64 || rt == MirType::F64 {
                MirType::F64
            } else {
                MirType::I64
            }
        }
        MirExpr::Binary { op, .. }
            if op == "Lt"
                || op == "Le"
                || op == "Gt"
                || op == "Ge"
                || op == "Eq"
                || op == "Ne"
                || op == "And"
                || op == "Or" =>
        {
            MirType::I64
        }
        MirExpr::Unary { op, expr } if op == "Neg" => {
            infer_mir_expr_type(expr, owner, function_returns, locals_ty)
        }
        MirExpr::Call { callee, args } => {
            if let MirExpr::Var(name) = &**callee {
                if let Some(ret) = function_returns.get(name) {
                    return ret.clone();
                }
                if name == "min" || name == "max" {
                    if args.iter().any(|a| {
                        infer_mir_expr_type(a, owner, function_returns, locals_ty) == MirType::F64
                    }) {
                        return MirType::F64;
                    }
                    return MirType::I64;
                }
            }
            if let Some((namespace, field)) = extract_stdlib_call_target_mir_static(callee.as_ref())
            {
                let ns_key = format!("__ns_call__{namespace}.{field}");
                if let Some(ret) = function_returns.get(&ns_key) {
                    return ret.clone();
                }
                if namespace == "convert"
                    && (field == "to_float"
                        || field == "parse_f64"
                        || field == "i64_to_f64"
                        || field == "f64_from_bits")
                {
                    return MirType::F64;
                }
                if namespace == "math" && field == "sqrt" {
                    return MirType::F64;
                }
                if namespace == "simd" && field == "f64x2_extract" {
                    return MirType::F64;
                }
                if namespace == "convert"
                    && (field == "to_str" || field == "to_str_f64" || field == "format_f64")
                {
                    return MirType::Str;
                }
                if namespace == "json.builder" && field == "finish" {
                    return MirType::Str;
                }
                if namespace == "json.builder" {
                    return MirType::JsonBuilder;
                }
            }
            MirType::I64
        }
        MirExpr::List(_) | MirExpr::Map(_) => MirType::Unknown,
        MirExpr::ResultOk { .. } | MirExpr::ResultErr { .. } => MirType::Result,
        _ => MirType::I64,
    }
}

fn collect_local_types(
    stmts: &[MirStmt],
    owner: &MirFunction,
    function_returns: &BTreeMap<String, MirType>,
    locals_ty: &mut BTreeMap<String, MirType>,
) {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { name, expr } | MirStmt::Assign { name, expr } => {
                let ty = infer_mir_expr_type(expr, owner, function_returns, locals_ty);
                locals_ty.insert(name.clone(), ty);
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_local_types(then_body, owner, function_returns, locals_ty);
                collect_local_types(else_body, owner, function_returns, locals_ty);
            }
            MirStmt::While { body, .. }
            | MirStmt::Repeat { body, .. }
            | MirStmt::For { body, .. } => {
                collect_local_types(body, owner, function_returns, locals_ty);
            }
            _ => {}
        }
    }
}

fn value_type_for_expr(
    expr: &MirExpr,
    owner: &MirFunction,
    function_returns: &BTreeMap<String, MirType>,
    ptr_ty: ir::Type,
    locals_ty: &BTreeMap<String, MirType>,
) -> ir::Type {
    match expr {
        MirExpr::Float(_) => ir::types::F64,
        MirExpr::Bool(_) => ir::types::I8,
        MirExpr::Str(_) => ptr_ty,
        MirExpr::Var(name) => {
            if let Some(ty) = locals_ty.get(name) {
                return mir_ty_to_clif(ty, ptr_ty);
            }
            if is_var_known_string_in_owner(owner, name) {
                ptr_ty
            } else if owner
                .params
                .iter()
                .any(|param| param.name == *name && param.ty == MirType::F64)
            {
                ir::types::F64
            } else if owner
                .params
                .iter()
                .any(|param| param.name == *name && param.ty == MirType::Bool)
            {
                ir::types::I8
            } else {
                ir::types::I64
            }
        }
        MirExpr::Binary { left, op, right } if op == "Add" => {
            if is_known_string_expr_with_owner(left, owner)
                || is_known_string_expr_with_owner(right, owner)
            {
                ptr_ty
            } else {
                let left_ty = value_type_for_expr(left, owner, function_returns, ptr_ty, locals_ty);
                let right_ty =
                    value_type_for_expr(right, owner, function_returns, ptr_ty, locals_ty);
                if left_ty == ir::types::F64 || right_ty == ir::types::F64 {
                    ir::types::F64
                } else {
                    ir::types::I64
                }
            }
        }
        MirExpr::Binary { left, op, right }
            if op == "Sub"
                || op == "Mul"
                || op == "Div"
                || op == "Lt"
                || op == "Le"
                || op == "Gt"
                || op == "Ge" =>
        {
            let left_ty = value_type_for_expr(left, owner, function_returns, ptr_ty, locals_ty);
            let right_ty = value_type_for_expr(right, owner, function_returns, ptr_ty, locals_ty);
            if op == "Lt" || op == "Le" || op == "Gt" || op == "Ge" {
                ir::types::I64
            } else if left_ty == ir::types::F64 || right_ty == ir::types::F64 {
                ir::types::F64
            } else {
                ir::types::I64
            }
        }
        MirExpr::List(_) | MirExpr::Map(_) => ptr_ty,
        MirExpr::Member { field, .. } if field == "len" => ir::types::I64,
        MirExpr::Index { .. } => ir::types::I64,
        MirExpr::Slice { object_is_str, .. } if *object_is_str => ptr_ty,
        MirExpr::Async { expr } | MirExpr::Await { expr } => {
            value_type_for_expr(expr, owner, function_returns, ptr_ty, locals_ty)
        }
        MirExpr::Call { callee, .. } if matches!(&**callee, MirExpr::Var(name) if name == "chan") => {
            ptr_ty
        }
        MirExpr::Call { callee, .. }
            if matches!(&**callee, MirExpr::Var(name)
                if name == "len"
                    || name == "cpu_count"
                    || name == "sorted_desc") =>
        {
            ir::types::I64
        }
        MirExpr::Call { callee, args } if matches!(&**callee, MirExpr::Var(name) if name == "min" || name == "max") => {
            if args.iter().any(|arg| {
                value_type_for_expr(arg, owner, function_returns, ptr_ty, locals_ty)
                    == ir::types::F64
            }) {
                ir::types::F64
            } else {
                ir::types::I64
            }
        }
        MirExpr::Call { callee, .. }
            if matches!(&**callee, MirExpr::Var(name)
                if matches!(function_returns.get(name), Some(MirType::Str))) =>
        {
            ptr_ty
        }
        MirExpr::Call { callee, .. }
            if matches!(&**callee, MirExpr::Var(name)
                if matches!(function_returns.get(name), Some(MirType::F64))) =>
        {
            ir::types::F64
        }
        MirExpr::Call { callee, .. }
            if {
                extract_stdlib_call_target_mir_static(callee.as_ref())
                    .and_then(|(ns, f)| function_returns.get(&format!("__ns_call__{ns}.{f}")))
                    .is_some()
            } =>
        {
            let (ns, f) = extract_stdlib_call_target_mir_static(callee.as_ref()).unwrap();
            let ret_ty = function_returns
                .get(&format!("__ns_call__{ns}.{f}"))
                .unwrap();
            mir_ty_to_clif(ret_ty, ptr_ty)
        }
        MirExpr::Call { callee, .. }
            if matches!(
                extract_stdlib_call_target_mir_static(callee.as_ref()),
                Some((ref namespace, ref field))
                    if (namespace == "convert" && (field == "to_float" || field == "parse_f64" || field == "i64_to_f64" || field == "f64_from_bits"))
                    || (namespace == "math" && field == "sqrt")
                        || (namespace == "simd" && field == "f64x2_extract")
            ) =>
        {
            ir::types::F64
        }
        MirExpr::Call { callee, .. }
            if matches!(&**callee, MirExpr::Var(name)
                if matches!(function_returns.get(name), Some(MirType::Bool))) =>
        {
            ir::types::I8
        }
        MirExpr::Call { callee, .. }
            if matches!(
                extract_stdlib_call_target_mir_static(callee.as_ref()),
                Some((ref namespace, ref field))
                    if (namespace == "json" && field == "from_map")
                        || (namespace == "json.builder" && field != "finish")
            ) =>
        {
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
        MirExpr::Call { callee, .. } if matches!(&**callee, MirExpr::Member { field, .. } if field == "sort_desc" || field == "take") => {
            ptr_ty
        }
        MirExpr::Call { callee, .. }
            if matches!(
                &**callee,
                MirExpr::Member { object, field, .. }
                if matches!(&**object, MirExpr::Var(namespace)
                    if (namespace == "path" && (field == "join" || field == "parent" || field == "basename"))
                    || (namespace == "fs" && field == "read_text")
                    || (namespace == "net" && (field == "read" || field == "resolve"))
                    || (namespace == "convert" && (field == "to_str" || field == "to_str_f64" || field == "format_f64"))
                    || (namespace == "text" && (field == "trim" || field == "replace" || field == "to_lower" || field == "to_upper" || field == "split_part"))
                    || (namespace == "encoding" && (field == "hex_encode" || field == "hex_decode" || field == "base64_encode" || field == "base64_decode" || field == "url_encode" || field == "url_decode"))
                    || (namespace == "env" && (field == "get" || field == "get_required"))
                    || (namespace == "cli" && field == "arg")
                    || (namespace == "json" && (field.starts_with("encode_") || field.starts_with("decode_") || field == "canonical" || field == "repeat_array"))
                    || (namespace == "regex" && field == "replace_all")
                    || (namespace == "http" && (field == "status_text" || field == "build_request_line" || field == "build_response" || field == "send" || field == "response" || field == "get" || field == "post" || field == "request"))
                    || (namespace == "hash" && field == "md5_hex")
                    || (namespace == "crypto" && field == "secp256k1_bench")
                    || (namespace == "math" && field == "edigits")
                )
            ) =>
        {
            ptr_ty
        }
        MirExpr::Unary { op, expr } if op == "Neg" => {
            value_type_for_expr(expr, owner, function_returns, ptr_ty, locals_ty)
        }
        MirExpr::Unary { .. } => ir::types::I64,
        MirExpr::Question { .. } => ir::types::I64,
        MirExpr::Old { expr } => {
            value_type_for_expr(expr, owner, function_returns, ptr_ty, locals_ty)
        }
        MirExpr::ResultOk { .. } | MirExpr::ResultErr { .. } => ptr_ty,
        MirExpr::DotResult => ir::types::I64,
        _ => ir::types::I64,
    }
}

fn mir_ty_to_clif(ty: &MirType, ptr_ty: ir::Type) -> ir::Type {
    match ty {
        MirType::I64 | MirType::Unknown => ir::types::I64,
        MirType::F64 => ir::types::F64,
        MirType::Bool => ir::types::I8,
        MirType::Str | MirType::Json | MirType::JsonBuilder | MirType::Result => ptr_ty,
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
                ..Default::default()
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
                ..Default::default()
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
                ..Default::default()
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
