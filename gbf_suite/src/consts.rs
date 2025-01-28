pub enum ExitCode {
    Success = 0,
    EnvVarNotFound = 1,
    InvalidDir = 2,
    FileReadError = 3,
    SvgConversionError = 4,
    FileWriteError = 5,
    JsonSerializeError = 6,
    UnexpectedError = 7,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

pub const GBF_SUITE_INPUT_DIR_ENV_VAR: &str = "GBF_SUITE_DIR";
pub const GBF_SUITE_OUTPUT_ENV_VAR: &str = "GBF_OUTPUT_DIR";

// Cloud vars
pub const GBF_AWS_BUCKET: &str = "gbf-rs";
pub const GBF_AWS_DYNAMO_VERSION_TABLE: &str = "gbf-rs-version";
pub const GBF_AWS_DYNAMO_GRAPHVIZ_TABLE: &str = "gbf-rs-graphviz";
pub const GBF_AWS_DYNAMO_MODULE_TABLE: &str = "gbf-rs-module";
pub const GBF_AWS_DYNAMO_FUNCTION_TABLE: &str = "gbf-rs-function";
pub const GBF_AWS_DYNAMO_FUNCTION_ERROR_TABLE: &str = "gbf-rs-function-error";
pub const GBF_AWS_REGION: &str = "us-east-1";
