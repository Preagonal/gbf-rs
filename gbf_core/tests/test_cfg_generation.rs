use common::{get_all_bytecode_files, load_bytecode};
use gbf_core::cfg_dot::{CfgDotConfig, DotRenderableGraph};
pub mod common;

#[test]
fn test_all_cfg_render() {
    for fname in get_all_bytecode_files().unwrap() {
        let reader = load_bytecode(&fname).unwrap();
        let module = gbf_core::module::ModuleBuilder::new()
            .name(fname.clone())
            .reader(Box::new(reader))
            .build()
            .unwrap();

        for function in module.iter() {
            let result = function.render_dot(CfgDotConfig::default());

            // for now, we check if it contains digraph
            assert!(result.contains("digraph"));
        }
    }
}
