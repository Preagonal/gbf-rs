#![feature(backtrace_frames)]

use std::{
    collections::HashMap,
    env,
    fmt::Write,
    fs::{self},
    path, process,
    sync::Mutex,
};

use consts::{ExitCode, GBF_SUITE_INPUT_DIR_ENV_VAR};
use dotenv::dotenv;
use gbf_core::{
    decompiler::{
        ast::visitors::emit_context::EmitContextBuilder,
        function_decompiler::FunctionDecompilerBuilder,
    },
    module::ModuleBuilder,
};

use rayon::prelude::*;
use std::sync::LazyLock;

pub mod consts;

static STATS: LazyLock<Mutex<DecompStats>> = LazyLock::new(|| Mutex::new(DecompStats::new()));

struct DecompStats {
    total_scripts: usize,
    total_functions: usize,
    successful_functions: usize,
    error_counts: HashMap<String, usize>,
}

impl DecompStats {
    fn new() -> Self {
        Self {
            total_scripts: 0,
            total_functions: 0,
            successful_functions: 0,
            error_counts: HashMap::new(),
        }
    }

    fn add_script(&mut self, function_count: usize) {
        self.total_scripts += 1;
        self.total_functions += function_count;
    }

    fn add_success(&mut self) {
        self.successful_functions += 1;
    }

    fn add_error(&mut self, error: String) {
        *self.error_counts.entry(error).or_insert(0) += 1;
    }

    fn log_stats(&mut self) {
        let coverage = if self.total_functions > 0 {
            (self.successful_functions as f32 / self.total_functions as f32) * 100.0
        } else {
            0.0
        };

        // Get top 5 errors
        let mut errors: Vec<_> = self.error_counts.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        let top_errors: Vec<_> = errors.iter().take(10).collect();

        log::info!(
            "Decompilation Statistics:\n\
            Total Scripts: {}\n\
            Total Functions: {}\n\
            Total Coverage: {:.1}%\n\
            Top 10 most common errors:{}",
            self.total_scripts,
            self.total_functions,
            coverage,
            if top_errors.is_empty() {
                "\nNone".to_string()
            } else {
                top_errors.iter().enumerate().fold(
                    String::new(),
                    |mut output, (i, (err, count))| {
                        let _ = write!(output, "\n{}. {} ({})", i + 1, err, count);
                        output
                    },
                )
            }
        );
    }
}

fn main() {
    // Load .env file if it exists
    dotenv().ok();

    let config_path = path::Path::new(env!("CARGO_MANIFEST_DIR")).join("logging_config.yaml");
    log4rs::init_file(config_path, Default::default()).unwrap();

    // Attempt to unwrap the environment variable. If it fails, exit with an error code.
    let value = env::var(GBF_SUITE_INPUT_DIR_ENV_VAR).unwrap_or_else(|_| {
        log::error!(
            "Environment variable {} not found",
            GBF_SUITE_INPUT_DIR_ENV_VAR
        );
        process::exit(ExitCode::EnvVarNotFound.into());
    });

    // Attempt to open the directory. If it fails, exit with an error code.
    let dir = std::fs::read_dir(value).unwrap_or_else(|_| {
        log::error!("Invalid directory");
        process::exit(ExitCode::InvalidDir.into());
    });

    // Process each file in the directory (each file is considered a module).
    dir.par_bridge().for_each(|entry| {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                log::error!("Failed to get the directory entry: {}", err);
                return;
            }
        };

        let result = process_module(entry.path().as_ref());
        if let Err(e) = result {
            log::error!("Failed to process module: {}", e);
            std::process::exit(ExitCode::UnexpectedError.into());
        }
    });

    // After processing all modules, log the final statistics.
    STATS.lock().unwrap().log_stats();
}

/// Processes a single module (file), building it and then decompiling each function.
/// Decompilation is performed in parallel for each function.
fn process_module(path: &path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let module_name = path
        .file_name()
        .ok_or("Failed to get file name")?
        .to_str()
        .ok_or("Failed to convert file name to string")?
        .to_string();

    let reader = fs::File::open(path)?;

    let module = match ModuleBuilder::new()
        .name(module_name.clone())
        .reader(Box::new(reader))
        .build()
    {
        Ok(module) => module,
        Err(e) => {
            log::error!("Failed to build module {}: {:?}", module_name, e);
            return Ok(()); // Skip this module rather than returning an error
        }
    };

    // We add the script to our stats (one script is the entire module).
    // Also note how many functions are in this module.
    STATS.lock().unwrap().add_script(module.len());

    // Decompile each function in parallel using rayon
    module.par_iter().for_each(|func| {
        let func_name = func.id.name.clone().unwrap_or_else(|| "entry".to_string());
        log::info!("Decompiling function {}", func_name);

        let mut decompiler = FunctionDecompilerBuilder::new(func.clone())
            .structure_analysis_max_iterations(100)
            .structure_debug_mode(true)
            .build();

        let res = decompiler.decompile(
            EmitContextBuilder::default()
                .include_ssa_versions(true)
                .build(),
        );

        match res {
            Ok(_) => {
                STATS.lock().unwrap().add_success();
                log::info!("Decompiled function {}", func_name);
            }
            Err(e) => {
                STATS.lock().unwrap().add_error(e.to_string());
                log::error!("Error decompiling function {}: {}", func_name, e);
            }
        }
    });

    Ok(())
}
