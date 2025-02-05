pub enum ExitCode {
    Success = 0,
    EnvVarNotFound = 1,
    InvalidDir = 2,
    FileReadError = 3,
    UnexpectedError = 4,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

pub const GBF_SUITE_INPUT_DIR_ENV_VAR: &str = "GBF_STATS_DIR";
