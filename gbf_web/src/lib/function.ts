// lib/getAllVersions.ts
import { cache } from 'react';
import { fetchAllFunctions, fetchFunctionByVersionAndId } from './dynamodb/function-repo';

// `cache` is a Next 13 helper that memoizes the result across the same server session
export const getAllFunctions = cache(async (versionId: string, moduleId: string) => {
    return await fetchAllFunctions(versionId, moduleId);
});

export const getFunction = cache(async (versionId: string, moduleId: string, functionId: number) => {
    return await fetchFunctionByVersionAndId(versionId, moduleId, functionId);
});