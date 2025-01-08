use common::{load_bytecode, load_expected_output};
use gbf_core::decompiler::{
    ast::visitors::emit_context::EmitContext, function_decompiler::FunctionDecompiler,
};
pub mod common;

#[test]
fn load_simple_gs2() {
    let reader = load_bytecode("simple.gs2bc").unwrap();
    let _expected = load_expected_output("simple.gs2")
        .unwrap()
        .trim()
        .to_string();

    let module = gbf_core::module::ModuleBuilder::new()
        .name("simple.gs2".to_string())
        .reader(Box::new(reader))
        .build()
        .unwrap();

    // Get the entry function
    let entry_function = module.get_entry_function();

    // Decompile the entry function
    let mut decompiler = FunctionDecompiler::new(entry_function.clone());
    let decompiled = decompiler.decompile(EmitContext::default());

    // For now, assert that the decompiler did not fail
    // TODO: We need to update the test to compare the decompiled output with the expected output
    // once the decompiler is more stable.
    assert!(decompiled.is_ok());
}
