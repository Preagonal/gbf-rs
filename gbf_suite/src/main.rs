use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{self, types::AttributeValue};
use aws_sdk_s3::Client;
use consts::{ExitCode, GBF_SUITE_INPUT_DIR_ENV_VAR};
use gbf_core::{
    cfg_dot::{CfgDotConfig, DotRenderableGraph},
    decompiler::{
        ast::visitors::emit_context::{EmitContextBuilder, EmitVerbosity, IndentStyle},
        function_decompiler::FunctionDecompiler,
    },
    module::ModuleBuilder,
    utils::VERSION,
};
use gbf_result::{GbfFunctionResult, GbfModuleResult, GbfSuiteResult, SvgRef};
use serde_dynamo::to_attribute_value;
use sha2::{Digest, Sha256};
use std::{
    env, error, fs,
    io::{Read, Write},
    path,
    process::{self},
    time::Instant,
};

pub mod consts;
pub mod gbf_result;

fn convert_graphviz_to_svg(dot: &str) -> String {
    let dot = dot.replace("\n", "");
    let dot = dot.replace("\\n", "\n");

    let mut command = process::Command::new("dot")
        .arg("-Tsvg")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            log::error!("Failed to spawn dot: {}", e);
            std::process::exit(ExitCode::SvgConversionError.into());
        });

    let stdin = command
        .stdin
        .as_mut()
        .ok_or("Failed to open stdin")
        .unwrap_or_else(|e| {
            log::error!("Failed to open stdin: {}", e);
            std::process::exit(ExitCode::SvgConversionError.into());
        });

    stdin.write_all(dot.as_bytes()).unwrap_or_else(|e| {
        log::error!("Failed to write to stdin: {}", e);
        std::process::exit(ExitCode::SvgConversionError.into());
    });

    let output = command.wait_with_output().unwrap_or_else(|e| {
        log::error!("Failed to wait for output: {}", e);
        std::process::exit(ExitCode::SvgConversionError.into());
    });

    if !output.stderr.is_empty() {
        log::error!(
            "dot command error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(ExitCode::SvgConversionError.into());
    }

    String::from_utf8(output.stdout).unwrap_or_else(|e| {
        log::error!("Failed to convert output to string: {}", e);
        std::process::exit(ExitCode::SvgConversionError.into());
    })
}

/// Hash a file as SHA256
fn hash_file(file: &path::Path) -> Result<String, Box<dyn error::Error>> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(file)?;
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn process_module(path: &path::Path) -> Result<GbfModuleResult, Box<dyn error::Error>> {
    let module_id = hash_file(path)?;

    let start_time = Instant::now();
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
            return Ok(GbfModuleResult {
                file_name: module_name.clone(),
                gbf_version: VERSION.to_string(),
                functions: Err(e),
                module_load_time: start_time.elapsed(),
                decompile_success: false,
                module_id: module_id.clone(),
            })
        }
    };
    let module_load_time = start_time.elapsed();

    let mut function_results: Vec<GbfFunctionResult> = Vec::new();

    let mut module_decompile_success = true;

    for function in module.iter() {
        let name = function.id.name.clone();

        let dot = function.render_dot(CfgDotConfig::default());
        let svg = convert_graphviz_to_svg(&dot);

        let context = EmitContextBuilder::default()
            .verbosity(EmitVerbosity::Pretty)
            .format_number_hex(true)
            .indent_style(IndentStyle::Allman)
            .include_ssa_versions(true)
            .build();

        let start_time = Instant::now();
        let mut decompiler = FunctionDecompiler::new(function.clone());
        let decompile_result = decompiler.decompile(context);
        let decompile_time = start_time.elapsed();

        if decompile_result.is_err() {
            module_decompile_success = false;
        }

        let decompile_success = decompile_result.is_ok();
        function_results.push(GbfFunctionResult {
            gbf_version: VERSION.to_string(),
            module_id: module_id.clone(),
            function_name: name,
            function_address: function.id.address,
            svg_cfg: SvgRef::Text(svg),
            decompile_result,
            decompile_success,
            decompile_time,
        });
    }

    Ok(GbfModuleResult {
        file_name: module_name.clone(),
        gbf_version: VERSION.to_string(),
        functions: Ok(function_results),
        module_load_time,
        decompile_success: module_decompile_success,
        module_id: module_id.clone(),
    })
}

pub fn hash_svg(svg: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(svg);
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[tokio::main]
async fn main() {
    let config_path = path::Path::new(env!("CARGO_MANIFEST_DIR")).join("logging_config.yaml");
    log4rs::init_file(config_path, Default::default()).unwrap();

    // Load AWS credentials
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

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

    let mut final_results: Vec<GbfModuleResult> = Vec::new();

    let time = Instant::now();

    let mut suite_decompile_success = true;

    // Iterate over the directory entries.
    for entry in dir {
        // Attempt to unwrap the entry. If it fails, log an error and continue.
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                log::error!("Failed to get the directory entry: {}", err);
                continue;
            }
        };

        let result = process_module(entry.path().as_ref());

        // Attempt to unwrap the result. If it fails, log an error and exit.
        let result = match result {
            Ok(result) => result,
            Err(err) => {
                log::error!("Failed to process module: {}", err);
                process::exit(ExitCode::FileReadError.into());
            }
        };

        if !result.decompile_success {
            suite_decompile_success = false;
        }

        final_results.push(result);
    }

    let total_suite_time = time.elapsed();

    // Store each SVG in the AWS S3 bucket
    let s3_client = Client::new(&aws_config);

    // For each module, store each function's SVG in the S3 bucket
    for module in final_results.iter_mut() {
        if let Ok(functions) = &mut module.functions {
            for function in functions.iter_mut() {
                let svg = match &function.svg_cfg {
                    SvgRef::Text(svg) => svg,
                    SvgRef::Key(_) => continue,
                };

                let hash = hash_svg(svg.clone());
                let key = format!(
                    "img/{}/{}/{}.svg",
                    module.module_id, function.function_address, hash
                );

                s3_client
                    .put_object()
                    .bucket(consts::GBF_AWS_BUCKET)
                    .content_type("image/svg+xml")
                    .key(key.clone())
                    .body(svg.clone().into_bytes().into())
                    .send()
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to store object: {}", e);
                        process::exit(ExitCode::FileWriteError.into());
                    });

                log::info!("Successfully stored object: {}", key);

                // Update the function's SVG to be a hash
                function.svg_cfg = SvgRef::Key(key);
            }
        }
    }

    let final_results = GbfSuiteResult {
        gbf_version: VERSION.to_string(),
        total_time: total_suite_time,
        modules: final_results,
        decompile_success: suite_decompile_success,
    };

    // Put data into the dynamo table, using the version as the primary key
    let dynamo_client = aws_sdk_dynamodb::Client::new(&aws_config);

    let map = serde_json::to_value(final_results).unwrap_or_else(|e| {
        log::error!("Failed to serialize results to JSON: {}", e);
        process::exit(ExitCode::JsonSerializeError.into());
    });

    let attr = to_attribute_value(map).unwrap_or_else(|e| {
        log::error!("Failed to convert to attribute value: {}", e);
        process::exit(ExitCode::JsonSerializeError.into());
    });

    let _put_item = dynamo_client
        .put_item()
        .table_name(consts::GBF_AWS_DYNAMO_TABLE)
        .item("version", AttributeValue::S(VERSION.to_string()))
        .item("module_json", attr)
        .send()
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to store object: {}", e);
            process::exit(ExitCode::FileWriteError.into());
        });

    log::info!("Successfully stored DynamoDB item for version {}", VERSION);
}
