// lib/getAllVersions.ts
import { cache } from 'react';
import { fetchAllModules, fetchModuleByVersionAndId } from './dynamodb/module-repo';

// `cache` is a Next 13 helper that memoizes the result across the same server session
export const getAllModules = cache(async (versionId: string) => {
    return await fetchAllModules(versionId);
});

export const getModule = cache(async (versionId: string, moduleId: string) => {
    return await fetchModuleByVersionAndId(versionId, moduleId);
});