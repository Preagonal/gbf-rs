import { cacheable } from './cache';
import { fetchAllFunctions, fetchFunctionByVersionAndId } from './dynamodb/function-repo';

export const getAllFunctions = cacheable.wrap(fetchAllFunctions, { keyPrefix: 'allfunctions' });
export const getFunction = cacheable.wrap(fetchFunctionByVersionAndId, { keyPrefix: 'function' });