import { fetchFunctionError } from './dynamodb/function-error-repo';
import { cacheable } from './cache';

export const getFunctionError = cacheable.wrap(fetchFunctionError, { keyPrefix: 'functionerror' });