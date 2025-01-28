// lib/getAllVersions.ts
import { cache } from 'react';
import { fetchAllVersions } from './dynamodb/version-repo';

// `cache` is a Next 13 helper that memoizes the result across the same server session
export const getAllVersions = cache(async () => {
    return await fetchAllVersions();
});
