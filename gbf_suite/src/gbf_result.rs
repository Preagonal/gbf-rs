use std::time::Duration;

use gbf_core::{
    decompiler::function_decompiler::FunctionDecompilerError, module::ModuleError,
    utils::Gs2BytecodeAddress,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SvgCfgType {
    SvgText,
    SvgUrl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SvgRef {
    Text(String),
    Key(String),
}

#[derive(Debug, Serialize)]
pub struct GbfFunctionResult {
    /// The GBF version used to decomplie the module.
    pub gbf_version: String,

    /// The module ID of the module this function belongs to.
    pub module_id: String,

    /// The name of the function.
    pub function_name: Option<String>,

    /// The address of the function (can be used as a unique identifier).
    pub function_address: Gs2BytecodeAddress,

    /// The SVG representation of the CFG.
    pub svg_cfg: SvgRef,

    /// If the decompilation was successful.
    pub decompile_success: bool,

    /// The decompilation result.
    pub decompile_result: Result<String, FunctionDecompilerError>,

    /// The time it took to decompile the function.
    pub decompile_time: Duration,
}

#[derive(Debug, Serialize)]
pub struct GbfModuleResult {
    /// The GBF version used to decomplie the module.
    pub gbf_version: String,

    /// The module ID of the module (SHA256 hash of the module).
    pub module_id: String,

    /// The file name of the module.
    pub file_name: String,

    /// The time it took to load the module.
    pub module_load_time: Duration,

    /// The list of functions in the module.
    pub functions: Result<Vec<GbfFunctionResult>, ModuleError>,

    /// If the decompilation was successful.
    pub decompile_success: bool,
}

#[derive(Debug, Serialize)]
pub struct GbfSuiteResult {
    /// The GBF version used to decomplie the module.
    pub gbf_version: String,

    /// The time it took to run the entire suite.
    pub total_time: Duration,

    /// The list of module results.
    pub modules: Vec<GbfModuleResult>,

    /// If decompliation was successful.
    pub decompile_success: bool,
}
