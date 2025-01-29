use std::time::Duration;

use gbf_core::{
    decompiler::function_decompiler::{FunctionDecompilerError, FunctionDecompilerErrorContext},
    utils::Gs2BytecodeAddress,
};
use serde::{Deserialize, Serialize};

/// The dynamodb entry for a GraphViz dot file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GbfGraphvizStructureAnalaysisDao {
    /// The version of the GBF used to generate the dot file.
    pub gbf_version: String,

    /// The module ID of the module (SHA256 hash of the module).
    pub module_id: String,

    /// The function address of the function (can be used as a unique identifier).
    pub function_address: Gs2BytecodeAddress,

    /// The structure analysis step.
    pub structure_analysis_step: usize,

    /// The S3 key of the dot file.
    pub dot_key: String,
}

impl GbfGraphvizStructureAnalaysisDao {
    pub fn pk_key(&self) -> String {
        "gbf_version#module_id#function_address".to_string()
    }

    pub fn pk_val(&self) -> String {
        format!(
            "{}#{}#{}",
            self.gbf_version, self.module_id, self.function_address
        )
    }

    pub fn dot_url(&self, bucket: &str) -> String {
        format!("https://{}.s3.amazonaws.com/{}", bucket, self.dot_key)
    }
}

/// The dynamodb entry for a GBF suite result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GbfVersionDao {
    /// The version of the GBF used.
    pub gbf_version: String,

    /// The total time it took to run the entire suite.
    pub total_time: Duration,

    /// Timestamp (e.g., "2025-01-28T12:34:56Z" or an epoch).
    pub suite_timestamp: u64, // or String for ISO8601
}

impl GbfVersionDao {
    pub fn pk_key(&self) -> String {
        "gbf_version".to_string()
    }

    pub fn pk_val(&self) -> String {
        self.gbf_version.clone()
    }
}

/// The dynamodb entry for a GBF function result.
#[derive(Debug, Serialize, Deserialize)]
pub struct GbfModuleDao {
    /// GBF version used to decompile the module.
    pub gbf_version: String,

    /// The module ID of the module (SHA256).
    pub module_id: String,

    /// The file name of the module.
    pub file_name: String,

    /// The time it took to load the module.
    pub module_load_time: Duration,

    /// If the module's decompilation was successful.
    pub decompile_success: bool,
}

impl GbfModuleDao {
    pub fn pk_key(&self) -> String {
        "gbf_version".to_string()
    }

    pub fn pk_val(&self) -> String {
        self.gbf_version.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GbfFunctionDao {
    /// The GBF version used to decompile the function.
    pub gbf_version: String,

    /// The module ID to which this function belongs.
    pub module_id: String,

    /// The function address (unique within the module).
    pub function_address: Gs2BytecodeAddress,

    /// The name of the function, if known.
    pub function_name: Option<String>,

    /// Whether the function was decompiled successfully.
    pub decompile_success: bool,

    /// The result of the decompilation attempt (could be an error).
    pub decompile_result: Option<String>,

    /// How long it took to decompile this function.
    pub total_time: Duration,

    /// The S3 key of the dot file.
    pub dot_key: String,
}

impl GbfFunctionDao {
    pub fn pk_key(&self) -> String {
        "gbf_version#module_id".to_string()
    }

    pub fn pk_val(&self) -> String {
        format!("{}#{}", self.gbf_version, self.module_id)
    }

    pub fn dot_url(&self, bucket: &str) -> String {
        format!("https://{}.s3.amazonaws.com/{}", bucket, self.dot_key)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct GbfFunctionErrorDao {
    /// GBF version
    pub gbf_version: String,

    /// Module ID
    pub module_id: String,

    /// The function address that encountered the error.
    pub function_address: Gs2BytecodeAddress,

    /// The type of error (e.g. structure analysis, parse error, etc.)
    pub error_type: String,

    /// A human-readable message or summary.
    pub message: String,

    /// A structured backtrace
    pub backtrace: GbfSimplifiedBacktrace,

    /// The context of the error.
    pub context: FunctionDecompilerErrorContext,
}

impl GbfFunctionErrorDao {
    pub fn pk_key(&self) -> String {
        "gbf_version#module_id".to_string()
    }

    pub fn pk_val(&self) -> String {
        format!("{}#{}", self.gbf_version, self.module_id)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GbfSimplifiedBacktrace {
    pub frames: Vec<GbfSimplifiedBacktraceFrame>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GbfSimplifiedBacktraceFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
}
