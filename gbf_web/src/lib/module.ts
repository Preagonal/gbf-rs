import { cacheable } from './cache';
import { fetchAllModules, fetchModuleByVersionAndId } from './dynamodb/module-repo';

export const getAllModules = cacheable.wrap(fetchAllModules, { keyPrefix: 'allmodules' });
export const getModule = cacheable.wrap(fetchModuleByVersionAndId, { keyPrefix: 'module' });