// lib/dynamodb/suite-repo.ts
import { GBF_AWS_DYNAMO_MODULE_TABLE } from '@/consts';
import { GbfModuleDao } from '@/dao/gbf-module-dao';
import { QueryCommand } from '@aws-sdk/lib-dynamodb';
import { DYNAMO_CLIENT } from './dynamo';

/**
 * Maps the dynamodb response to a GbfVersionDao.
 */
interface DynamoDBItem {
    gbf_version: string;
    module_id: string;
    file_name: string;
    module_load_time: number;
    decompile_success: boolean;
}

function mapToGbfModuleDao(item: unknown): GbfModuleDao {
    const dynamoDBItem = item as DynamoDBItem;
    return new GbfModuleDao({
        gbfVersion: dynamoDBItem.gbf_version,
        moduleId: dynamoDBItem.module_id,
        fileName: dynamoDBItem.file_name,
        moduleLoadTimeMillis: dynamoDBItem.module_load_time,
        decompileSuccess: dynamoDBItem.decompile_success
    });
}

/**
 * Fetches all modules from the GbfModuleResult table.
 */
export async function fetchAllModules(version: string): Promise<GbfModuleDao[]> {
    const params = {
        TableName: GBF_AWS_DYNAMO_MODULE_TABLE,
        KeyConditionExpression: 'gbf_version = :version',
        ExpressionAttributeValues: {
            ':version': version,
        },
    };
    const command = new QueryCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    return results.Items?.map(mapToGbfModuleDao) || [];
}

/**
 * Fetches a single module by version and module id.
 */
export async function fetchModuleByVersionAndId(version: string, moduleId: string): Promise<GbfModuleDao | null> {
    const params = {
        TableName: GBF_AWS_DYNAMO_MODULE_TABLE,
        KeyConditionExpression: 'gbf_version = :version AND module_id = :moduleId',
        ExpressionAttributeValues: {
            ':version': version,
            ':moduleId': moduleId,
        },
    };
    const command = new QueryCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    return results.Items?.map(mapToGbfModuleDao)[0] || null;
}