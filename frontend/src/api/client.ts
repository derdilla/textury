import createClient from 'openapi-fetch';
import type { paths } from '@/generated/api';

export const API_ROOT = '/api/v1'

export const api = createClient<paths>({
  baseUrl: API_ROOT,
  credentials: 'include',
});