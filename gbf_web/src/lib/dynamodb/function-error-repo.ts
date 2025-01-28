// lib/dynamodb/suite-repo.ts
import { GBF_AWS_DYNAMO_FUNCTION_ERROR_TABLE } from '@/consts';
import { GbfFunctionErrorDao, GbfSimplifiedBacktrace } from '@/dao/gbf-function-error-dao';
import { QueryCommand } from '@aws-sdk/lib-dynamodb';
import { DYNAMO_CLIENT } from './dynamo';

/**
 * Maps the dynamodb response to a GbfFunctionDao.
 */
interface DynamoDBItem {
    gbf_version: string;
    module_id: string;
    function_address: number;
    error_type: string;
    message: string;
    backtrace: GbfSimplifiedBacktrace;
}

function mapToGbfFunctionError(item: unknown): GbfFunctionErrorDao {
    const dynamoDBItem = item as DynamoDBItem;
    return new GbfFunctionErrorDao({
        gbfVersion: dynamoDBItem.gbf_version,
        moduleId: dynamoDBItem.module_id,
        functionAddress: dynamoDBItem.function_address,
        errorType: dynamoDBItem.error_type,
        message: dynamoDBItem.message,
        backtrace: dynamoDBItem.backtrace,
    });
}

/**
 * Fetches all function errors from the GbfFunction table.
 */
export async function fetchFunctionError(version: string, moduleId: string, function_address: number): Promise<GbfFunctionErrorDao | null> {
    const params = {
        TableName: GBF_AWS_DYNAMO_FUNCTION_ERROR_TABLE,
        KeyConditionExpression: `#pk = :versionModuleId AND function_address = :function_address`,
        ExpressionAttributeNames: {
            '#pk': GbfFunctionErrorDao.pkKey(),
        },
        ExpressionAttributeValues: {
            ':versionModuleId': `${version}#${moduleId}`,
            ':function_address': function_address,
        },
    };
    const command = new QueryCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    return results.Items?.map(mapToGbfFunctionError)[0] || null;
}
