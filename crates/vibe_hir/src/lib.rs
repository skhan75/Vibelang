use std::collections::BTreeSet;

use vibe_ast::TypeRef;

#[derive(Debug, Clone, Default)]
pub struct HirProgram {
    pub functions: Vec<HirFunction>,
}

#[derive(Debug, Clone, Default)]
pub struct HirFunction {
    pub name: String,
    pub is_public: bool,
    pub params: Vec<HirParam>,
    pub return_type: Option<TypeRef>,
    pub inferred_return_type: Option<String>,
    pub effects_declared: BTreeSet<String>,
    pub effects_observed: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HirParam {
    pub name: String,
    pub ty: Option<TypeRef>,
}

pub fn verify_hir(program: &HirProgram) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for f in &program.functions {
        if f.name.trim().is_empty() {
            return Err("empty function name in HIR".to_string());
        }
        if !seen.insert(f.name.clone()) {
            return Err(format!("duplicate function `{}` in HIR", f.name));
        }
    }
    Ok(())
}
