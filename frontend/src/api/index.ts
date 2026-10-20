import createClient from 'openapi-fetch';
import type { paths } from './generated/schema';

export const API_ROOT = '/api/v1';

export const api = createClient<paths>({
  baseUrl: API_ROOT,
  credentials: 'include',
});
