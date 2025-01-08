use common::{load_bytecode, load_expected_output};
use gbf_core::decompiler::{
    ast::visitors::emit_context::EmitContext, function_decompiler::FunctionDecompiler,
};
pub mod common;

#[test]
fn load_simple_gs2() {
    let reader = load_bytecode("simple.gs2bc").unwrap();
    let expected = load_expected_output("simple.gs2")
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
    let decompiled = decompiler
        .decompile(EmitContext::default())
        .unwrap()
        .trim()
        .to_string();

    // Compare the decompiled output with the expected output
    assert_eq!(decompiled, expected);
}
