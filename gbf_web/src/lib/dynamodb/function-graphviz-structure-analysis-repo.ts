// lib/dynamodb/suite-repo.ts
import { DYNAMO_DB_REGION, GBF_AWS_DYNAMO_GRAPHVIZ_TABLE } from '@/consts';
import { GbfGraphvizStructureAnalysisDao } from '@/dao/gbf-graphviz-structure-analysis-dao';
import { DynamoDBClient } from '@aws-sdk/client-dynamodb';
import { DynamoDBDocumentClient, QueryCommand } from '@aws-sdk/lib-dynamodb';

const client = new DynamoDBClient({ region: DYNAMO_DB_REGION });
const docClient = DynamoDBDocumentClient.from(client);

/**
 * Maps the dynamodb response to a GbfFunctionDao.
 */
interface DynamoDBItem {
    gbf_version: string;
    module_id: string;
    function_address: number;
    structure_analysis_step: number;
    dot_url: string;
}

function mapToGbfGraphvizStructureAnalaysisDao(item: unknown): GbfGraphvizStructureAnalysisDao {
    const dynamoDBItem = item as DynamoDBItem;
    return new GbfGraphvizStructureAnalysisDao({
        gbfVersion: dynamoDBItem.gbf_version,
        moduleId: dynamoDBItem.module_id,
        functionAddress: dynamoDBItem.function_address,
        structureAnalysisStep: dynamoDBItem.structure_analysis_step,
        dotUrl: dynamoDBItem.dot_url,
    });
}

/**
 * Fetches all graphviz structure analysis from the GbfGraphvizStructureAnalysis table.
    */
export async function fetchAllGraphvizStructureAnalysis(version: string, moduleId: string, functionAddress: number): Promise<GbfGraphvizStructureAnalysisDao[]> {
    const params = {
        TableName: GBF_AWS_DYNAMO_GRAPHVIZ_TABLE,
        KeyConditionExpression: `#pk = :versionModuleIdFunctionAddress`,
        ExpressionAttributeNames: {
            '#pk': GbfGraphvizStructureAnalysisDao.pkKey(),
        },
        ExpressionAttributeValues: {
            ':versionModuleIdFunctionAddress': `${version}#${moduleId}#${functionAddress}`,
        },
        SortBy: 'structure_analysis_step',
    };
    const command = new QueryCommand(params);
    const results = await docClient.send(command);
    return results.Items?.map(mapToGbfGraphvizStructureAnalaysisDao) || [];
}