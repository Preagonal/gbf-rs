use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::aws_sdk_dynamodb_1::to_attribute_value;

use crate::{
    consts::{self, GBF_AWS_BUCKET},
    gbf_result::{
        GbfFunctionDao, GbfFunctionErrorDao, GbfGraphvizStructureAnalaysisDao, GbfModuleDao,
        GbfVersionDao,
    },
    utils::hash_string,
};

pub struct AwsUpload {
    pub s3_client: aws_sdk_s3::Client,
    pub dynamo_client: aws_sdk_dynamodb::Client,
}

impl AwsUpload {
    pub async fn new() -> Self {
        // Load AWS credentials
        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let s3_client = aws_sdk_s3::Client::new(&sdk_config);
        let dynamo_client = aws_sdk_dynamodb::Client::new(&sdk_config);
        Self {
            s3_client,
            dynamo_client,
        }
    }

    pub async fn upload_graphviz_dot(
        &self,
        graphviz_dot: String,
    ) -> Result<String, aws_sdk_s3::Error> {
        // Upload the file to S3
        let s3_key = format!("graphviz/{}.dot", hash_string(&graphviz_dot));

        self.s3_client
            .put_object()
            .bucket(consts::GBF_AWS_BUCKET)
            .content_type("text/plain")
            .key(s3_key.clone())
            .body(graphviz_dot.into_bytes().into())
            .send()
            .await?;
        Ok(s3_key)
    }

    pub async fn upload_gbf_version(
        &self,
        gbf_version: GbfVersionDao,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        self.dynamo_client
            .put_item()
            .table_name(consts::GBF_AWS_DYNAMO_VERSION_TABLE)
            .item(
                gbf_version.pk_key(),
                AttributeValue::S(gbf_version.pk_val()),
            )
            .item("gbf_version", AttributeValue::S(gbf_version.gbf_version))
            .item(
                "total_time",
                AttributeValue::N(gbf_version.total_time.as_millis().to_string()),
            )
            .item(
                "suite_timestamp",
                AttributeValue::N(gbf_version.suite_timestamp.to_string()),
            )
            .send()
            .await?;
        Ok(())
    }

    pub async fn upload_gbf_module(
        &self,
        gbf_module: GbfModuleDao,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        self.dynamo_client
            .put_item()
            .table_name(consts::GBF_AWS_DYNAMO_MODULE_TABLE)
            .item(gbf_module.pk_key(), AttributeValue::S(gbf_module.pk_val()))
            .item("gbf_version", AttributeValue::S(gbf_module.gbf_version))
            .item("module_id", AttributeValue::S(gbf_module.module_id))
            .item("file_name", AttributeValue::S(gbf_module.file_name))
            .item(
                "module_load_time",
                AttributeValue::N(gbf_module.module_load_time.as_millis().to_string()),
            )
            .item(
                "decompile_success",
                AttributeValue::Bool(gbf_module.decompile_success),
            )
            .send()
            .await?;
        Ok(())
    }

    pub async fn upload_gbf_function(
        &self,
        gbf_function: GbfFunctionDao,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        self.dynamo_client
            .put_item()
            .table_name(consts::GBF_AWS_DYNAMO_FUNCTION_TABLE)
            .item(
                gbf_function.pk_key(),
                AttributeValue::S(gbf_function.pk_val()),
            )
            .item(
                "gbf_version",
                AttributeValue::S(gbf_function.clone().gbf_version),
            )
            .item(
                "module_id",
                AttributeValue::S(gbf_function.clone().module_id),
            )
            .item(
                "function_address",
                AttributeValue::N(gbf_function.clone().function_address.to_string()),
            )
            .item(
                "function_name",
                AttributeValue::S(
                    gbf_function
                        .clone()
                        .function_name
                        .unwrap_or("entry".to_string()),
                ),
            )
            .item(
                "decompile_success",
                AttributeValue::Bool(gbf_function.clone().decompile_success),
            )
            .item(
                "decompile_result",
                AttributeValue::S(
                    gbf_function
                        .clone()
                        .decompile_result
                        .unwrap_or("".to_string()),
                ),
            )
            .item(
                "total_time",
                AttributeValue::N(gbf_function.clone().total_time.as_millis().to_string()),
            )
            .item(
                "dot_url",
                AttributeValue::S(gbf_function.dot_url(GBF_AWS_BUCKET)),
            )
            .send()
            .await?;
        Ok(())
    }

    pub async fn upload_gbf_function_error(
        &self,
        gbf_function_error: GbfFunctionErrorDao,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backtrace = serde_json::to_value(gbf_function_error.clone().backtrace)?;
        let attr = to_attribute_value(backtrace)?;

        self.dynamo_client
            .put_item()
            .table_name(consts::GBF_AWS_DYNAMO_FUNCTION_ERROR_TABLE)
            .item(
                gbf_function_error.pk_key(),
                AttributeValue::S(gbf_function_error.pk_val()),
            )
            .item(
                "gbf_version",
                AttributeValue::S(gbf_function_error.gbf_version),
            )
            .item("module_id", AttributeValue::S(gbf_function_error.module_id))
            .item(
                "function_address",
                AttributeValue::N(gbf_function_error.function_address.to_string()),
            )
            .item(
                "error_type",
                AttributeValue::S(gbf_function_error.error_type),
            )
            .item("message", AttributeValue::S(gbf_function_error.message))
            .item("backtrace", attr)
            .send()
            .await?;
        Ok(())
    }

    pub async fn upload_gbf_graphviz_dao(
        &self,
        graphviz: GbfGraphvizStructureAnalaysisDao,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        self.dynamo_client
            .put_item()
            .table_name(consts::GBF_AWS_DYNAMO_GRAPHVIZ_TABLE)
            .item(graphviz.pk_key(), AttributeValue::S(graphviz.pk_val()))
            .item(
                "gbf_version",
                AttributeValue::S(graphviz.clone().gbf_version),
            )
            .item("module_id", AttributeValue::S(graphviz.clone().module_id))
            .item(
                "function_address",
                AttributeValue::N(graphviz.function_address.to_string()),
            )
            .item(
                "structure_analysis_step",
                AttributeValue::N(graphviz.structure_analysis_step.to_string()),
            )
            .item(
                "dot_url",
                AttributeValue::S(graphviz.dot_url(GBF_AWS_BUCKET)),
            )
            .send()
            .await?;
        Ok(())
    }
}
