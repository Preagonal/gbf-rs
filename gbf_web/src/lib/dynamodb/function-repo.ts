// lib/dynamodb/suite-repo.ts
import { GBF_AWS_DYNAMO_FUNCTION_TABLE } from '@/consts';
import { GbfFunctionDao } from '@/dao/gbf-function-dao';
import { QueryCommand } from '@aws-sdk/lib-dynamodb';
import { DYNAMO_CLIENT } from './dynamo';

/**
 * Maps the dynamodb response to a GbfFunctionDao.
 */
interface DynamoDBItem {
    gbf_version: string;
    module_id: string;
    function_address: number;
    function_name: string;
    decompile_success: boolean;
    decompile_result: string;
    total_time: number;
    dot_url: string;
}

function mapToGbfFunctionDao(item: unknown): GbfFunctionDao {
    const dynamoDBItem = item as DynamoDBItem;
    return new GbfFunctionDao({
        gbfVersion: dynamoDBItem.gbf_version,
        moduleId: dynamoDBItem.module_id,
        functionAddress: dynamoDBItem.function_address,
        functionName: dynamoDBItem.function_name,
        decompileSuccess: dynamoDBItem.decompile_success,
        totalTimeMillis: dynamoDBItem.total_time,
        decompileResult: dynamoDBItem.decompile_result,
        dotUrl: dynamoDBItem.dot_url,
    });
}

/**
 * Fetches all functions from the GbfFunction table.
 */
export async function fetchAllFunctions(version: string, moduleId: string): Promise<GbfFunctionDao[]> {
    const params = {
        TableName: GBF_AWS_DYNAMO_FUNCTION_TABLE,
        KeyConditionExpression: `#pk = :versionModuleId`,
        ExpressionAttributeNames: {
            '#pk': GbfFunctionDao.pkKey(),
        },
        ExpressionAttributeValues: {
            ':versionModuleId': `${version}#${moduleId}`,
        },
    };
    const command = new QueryCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    return results.Items?.map(mapToGbfFunctionDao) || [];
}

/**
 * Fetches a single module by version and module id.
 */
export async function fetchFunctionByVersionAndId(version: string, moduleId: string, functionAddress: number): Promise<GbfFunctionDao | null> {
    const params = {
        TableName: GBF_AWS_DYNAMO_FUNCTION_TABLE,
        KeyConditionExpression: `#pk = :versionModuleId AND function_address = :functionAddress`,
        ExpressionAttributeNames: {
            '#pk': GbfFunctionDao.pkKey(),
        },
        ExpressionAttributeValues: {
            ':versionModuleId': `${version}#${moduleId}`,
            ':functionAddress': functionAddress,
        },
    };
    const command = new QueryCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    return results.Items?.map(mapToGbfFunctionDao)[0] || null;
}