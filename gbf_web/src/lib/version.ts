import { fetchAllVersions } from './dynamodb/version-repo';
import { cacheable } from './cache';


export const getAllVersions = cacheable.wrap(fetchAllVersions, { keyPrefix: 'allversions' });