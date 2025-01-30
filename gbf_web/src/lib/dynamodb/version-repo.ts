// lib/dynamodb/suite-repo.ts
import { GBF_AWS_DYNAMO_VERSION_TABLE, MAX_DYNAMO_DB_BATCH_SIZE } from '@/consts';
import { GbfVersionDao } from '@/dao/gbf-version-dao';
import { QueryCommand, ScanCommand } from '@aws-sdk/lib-dynamodb';
import { DYNAMO_CLIENT } from './dynamo';
import semver from 'semver';

/**
 * Maps the dynamodb response to a GbfVersionDao.
 */
interface DynamoDBItem {
    gbf_version: string;
    total_time: number;
    suite_timestamp: number;
}

function mapToGbfVersionDao(item: unknown): GbfVersionDao {
    const dynamoDBItem = item as DynamoDBItem;
    return new GbfVersionDao({
        gbfVersion: dynamoDBItem.gbf_version,
        totalTimeMillis: dynamoDBItem.total_time,
        suiteTimestamp: dynamoDBItem.suite_timestamp,
    });
}

/**
 * Fetches all suites from the GbfSuiteResult table.
 */
export async function fetchAllVersions(): Promise<GbfVersionDao[]> {
    console.log("Fetching versions");
    const params = {
        TableName: GBF_AWS_DYNAMO_VERSION_TABLE,
        Limit: MAX_DYNAMO_DB_BATCH_SIZE,
    };
    const command = new ScanCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    const versions = results.Items?.map(mapToGbfVersionDao) || [];
    return versions.sort((a, b) => {
        if (semver.valid(a.gbfVersion) && semver.valid(b.gbfVersion)) {
            return semver.rcompare(a.gbfVersion, b.gbfVersion);
        }
        return a.gbfVersion.localeCompare(b.gbfVersion);
    });
}

/**
 * Fetches a single suite by version.
 */
export async function fetchSuiteByVersion(version: string): Promise<GbfVersionDao | null> {
    const params = {
        TableName: GBF_AWS_DYNAMO_VERSION_TABLE,
        KeyConditionExpression: 'gbf_version = :version',
        ExpressionAttributeValues: {
            ':version': version,
        },
    };
    const command = new QueryCommand(params);
    const results = await DYNAMO_CLIENT.send(command);
    return results.Items?.map(mapToGbfVersionDao)[0] || null;
}
