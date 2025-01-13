import { DYNAMO_DB_REGION, DYNAMO_DB_TABLE } from '@/consts';
import { DynamoDBClient } from '@aws-sdk/client-dynamodb';
import { DynamoDBDocumentClient, ScanCommand, ScanCommandInput, ScanCommandOutput } from "@aws-sdk/lib-dynamodb"


import fs from 'fs';
import path from 'path';
import { DEBUG } from '@/consts';
import { GbfSuiteResult, parseGbfSuiteResults } from './gbf-suite-result-dao';

async function fetchDynamoDbData() {
    const cacheFilePath = path.resolve(__dirname, 'cache.json');

    if (DEBUG && fs.existsSync(cacheFilePath)) {
        try {
            const cachedData = fs.readFileSync(cacheFilePath, 'utf-8');
            return JSON.parse(cachedData);
        } catch (err) {
            console.error("Error reading cache file:", err);
        }
    }

    const client = new DynamoDBClient({ region: DYNAMO_DB_REGION });
    const ddbDocClient = DynamoDBDocumentClient.from(client);

    const params: ScanCommandInput = {
        TableName: DYNAMO_DB_TABLE,
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const items: any[] = [];
    try {
        let ExclusiveStartKey;
        do {
            const data: ScanCommandOutput = await ddbDocClient.send(
                new ScanCommand({ ...params, ExclusiveStartKey })
            );
            if (data.Items) {
                items.push(...data.Items);
            }
            ExclusiveStartKey = data.LastEvaluatedKey; // Continue if more data is available
        } while (ExclusiveStartKey);

        if (DEBUG) {
            try {
                const cacheDir = path.dirname(cacheFilePath);
                if (!fs.existsSync(cacheDir)) {
                    fs.mkdirSync(cacheDir, { recursive: true });
                }
                fs.writeFileSync(cacheFilePath, JSON.stringify(items, null, 2), 'utf-8');
            } catch (err) {
                console.error("Error writing to cache file:", err);
            }
        }

        return items;
    } catch (err) {
        console.error("Error fetching data from DynamoDB:", err);
        throw err;
    }
}

export default async function fetchGbfSuiteResult(): Promise<GbfSuiteResult[]> {
    return fetchDynamoDbData().then((data) => {
        return parseGbfSuiteResults(data);
    });
}