// lib/getAllVersions.ts
import { cache } from 'react';
import { fetchFunctionError } from './dynamodb/function-error-repo';

export const getFunctionError = cache(async (versionId: string, moduleId: string, functionId: number) => {
    return await fetchFunctionError(versionId, moduleId, functionId);
});