#![feature(backtrace_frames)]

use std::{
    backtrace::Backtrace,
    env,
    fs::{self},
    path, process,
    time::Instant,
};

use aws_upload::AwsUpload;
use consts::{ExitCode, GBF_SUITE_INPUT_DIR_ENV_VAR};
use dotenv::dotenv;
use gbf_core::{
    cfg_dot::{CfgDotConfig, DotRenderableGraph},
    decompiler::{
        ast::visitors::emit_context::EmitContextBuilder,
        function_decompiler::{FunctionDecompilerBuilder, FunctionDecompilerErrorDetails},
    },
    module::ModuleBuilder,
    utils::VERSION,
};
use gbf_result::{
    GbfFunctionDao, GbfFunctionErrorDao, GbfGraphvizStructureAnalaysisDao, GbfModuleDao,
    GbfSimplifiedBacktrace, GbfSimplifiedBacktraceFrame, GbfVersionDao,
};
use regex::Regex;
use utils::hash_file;

pub mod aws_upload;
pub mod consts;
pub mod gbf_result;
pub mod utils;

#[tokio::main]
async fn main() {
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

    let uploader = aws_upload::AwsUpload::new().await;

    // If GBF_VERSION is set, create option and pass it to `process_module`
    let gbf_version_override = env::var("GBF_VERSION").ok();

    // Iterate over the directory entries.
    let time = Instant::now();
    for entry in dir {
        // Attempt to unwrap the entry. If it fails, log an error and continue.
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                log::error!("Failed to get the directory entry: {}", err);
                continue;
            }
        };

        let result = process_module(
            &uploader,
            entry.path().as_ref(),
            gbf_version_override.clone(),
        )
        .await;

        if let Err(e) = result {
            log::error!("Failed to process module: {}", e);
            std::process::exit(ExitCode::UnexpectedError.into());
        }
    }
    let total_time = time.elapsed();

    let gbf_version = GbfVersionDao {
        gbf_version: gbf_version_override.clone().unwrap_or(VERSION.to_string()),
        total_time,
        suite_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    match uploader.upload_gbf_version(gbf_version).await {
        Ok(_) => log::info!("Uploaded GBF version"),
        Err(e) => {
            log::error!("Failed to upload GBF version: {}", e);
            std::process::exit(ExitCode::UnexpectedError.into());
        }
    }
}

async fn process_module(
    uploader: &AwsUpload,
    path: &path::Path,
    gbf_version_override: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_name = path
        .file_name()
        .ok_or("Failed to get file name")?
        .to_str()
        .ok_or("Failed to convert file name to string")?
        .to_string();

    let module_id = hash_file(path)?;

    let reader = fs::File::open(path)?;

    let time = Instant::now();
    let module = match ModuleBuilder::new()
        .name(module_name.clone())
        .reader(Box::new(reader))
        .build()
    {
        Ok(module) => module,
        Err(e) => {
            log::error!("Failed to build module {}: {:?}", module_name, e);
            return Ok(());
        }
    };
    let module_time = time.elapsed();

    let mut module_dao = GbfModuleDao {
        gbf_version: gbf_version_override.clone().unwrap_or(VERSION.to_string()),
        module_id: module_id.to_string(),
        file_name: module_name,
        module_load_time: module_time,
        decompile_success: true,
    };

    for func in module.iter() {
        let func_basic_block_dot = func.render_dot(CfgDotConfig::default());
        let func_basic_block_dot_key = uploader.upload_graphviz_dot(func_basic_block_dot).await?;

        let time = Instant::now();

        let mut decompiler = FunctionDecompilerBuilder::new(func.clone())
            .structure_analysis_max_iterations(100)
            .structure_debug_mode(true)
            .build();

        let res = decompiler.decompile(
            EmitContextBuilder::default()
                .include_ssa_versions(true)
                .build(),
        );
        let function_time = time.elapsed();

        let decompile_success = if res.is_err() {
            let error = GbfFunctionErrorDao {
                gbf_version: gbf_version_override.clone().unwrap_or(VERSION.to_string()),
                module_id: module_id.to_string(),
                function_address: func.id.address,
                error_type: res.as_ref().unwrap_err().error_type().to_string(),
                message: res.as_ref().unwrap_err().to_string(),
                backtrace: process_backtrace(res.as_ref().unwrap_err().backtrace()),
                context: res.as_ref().unwrap_err().context().clone(),
            };
            module_dao.decompile_success = false;
            uploader.upload_gbf_function_error(error).await?;
            false
        } else {
            true
        };

        let function_dao = GbfFunctionDao {
            gbf_version: gbf_version_override.clone().unwrap_or(VERSION.to_string()),
            module_id: module_id.to_string(),
            function_address: func.id.address,
            function_name: func.id.name.clone(),
            decompile_success,
            total_time: function_time,
            dot_key: func_basic_block_dot_key,
            decompile_result: res.ok().map(|r| r.to_string()),
        };

        uploader.upload_gbf_function(function_dao).await?;

        log::info!(
            "Decompiled function {} in {}ms",
            func.id.name.clone().unwrap_or("entry".to_string()),
            function_time.as_millis()
        );

        // Upload all of the intermediate dot files
        for (i, dot) in decompiler
            .get_structure_analysis_snapshots()?
            .iter()
            .enumerate()
        {
            let dot_key = uploader.upload_graphviz_dot(dot.clone()).await?;

            let graphviz_dao = GbfGraphvizStructureAnalaysisDao {
                gbf_version: gbf_version_override.clone().unwrap_or(VERSION.to_string()),
                module_id: module_id.to_string(),
                function_address: func.id.address,
                structure_analysis_step: i,
                dot_key,
            };

            uploader.upload_gbf_graphviz_dao(graphviz_dao).await?;
        }
    }

    uploader.upload_gbf_module(module_dao).await?;

    Ok(())
}

pub fn process_backtrace(backtrace: &Backtrace) -> GbfSimplifiedBacktrace {
    // Filter frames to include only those starting with "gbf_core"
    let include_fn_regex = Regex::new(r"^gbf_core::").unwrap();

    let mut simplified_frames = Vec::new();

    for frame in backtrace.frames() {
        // Convert the frame to a string representation
        let frame_str = format!("{:?}", frame);

        // Extract the function name
        let function_name = frame_str
            .split_once("fn: \"")
            .and_then(|(_, rest)| rest.split_once("\""))
            .map(|(fn_name, _)| fn_name.to_string());

        // Extract the file path
        let file_path = frame_str
            .split_once("file: \"")
            .and_then(|(_, rest)| rest.split_once("\""))
            .map(|(file, _)| file.to_string());

        // Extract the line number
        let line_number = frame_str
            .split_once("line: ")
            .and_then(|(_, rest)| rest.split_once("}"))
            .and_then(|(line, _)| line.trim().parse::<u32>().ok());

        // Only include frames that match the `gbf_core` prefix
        if let Some(function_name) = function_name {
            if include_fn_regex.is_match(&function_name) {
                simplified_frames.push(GbfSimplifiedBacktraceFrame {
                    function: function_name,
                    file: file_path.unwrap(),
                    line: line_number.unwrap(),
                });
            }
        }
    }

    GbfSimplifiedBacktrace {
        frames: simplified_frames,
    }
}
