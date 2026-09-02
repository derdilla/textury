export const API_ROOT = '/api/v1'

export async function login(request: LoginRequest): Promise<PrivateAccountInfo | null> {
  const response = await fetch(`${API_ROOT}/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(request),
  });
  if (response.ok) return await response.json();
  return null;
}

export async function register(request: CreateAccountRequest) {
  const response = await fetch(`${API_ROOT}/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(request),
  });
  return response.ok
}

export async function whoami(): Promise<PrivateAccountInfo | null> {
  const response = await fetch(`${API_ROOT}/login`);
  if (response.ok) {
    return await response.json();
  }
  return null;
}

export async function logout(): Promise<boolean> {
  const response = await fetch(`${API_ROOT}/logout`, {
    method: 'POST',
  });
  return response.ok;
}

export async function invite(): Promise<UserInvite | null> {
  const response = await fetch(`${API_ROOT}/invite`);
  if (response.ok) return await response.json();
  return null;
}


export async function listProjects(): Promise<ProjectOverviewIndex | null> {
  const response = await fetch(`${API_ROOT}/projects`);
  if (response.ok) {
    const data: ProjectOverviewIndex = await response.json();
    return data;
  }
  return null;
}

export async function createProject(projectId: string, title: string) {
  projectId = projectId.replaceAll(' ', '_')
    .replaceAll(/([^a-zA-Z0-9.-_~!$&'()*+,;=:@])/g, '')
  const response = await fetch(`${API_ROOT}/project/${projectId}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      title: title,
    }),
  });
  return {
    success: response.ok,
    message: response.statusText,
  };
}

export async function deleteProject(projectId: string): Promise<boolean> {
  const response = await fetch(`${API_ROOT}/project/${projectId}`, {
    method: 'DELETE'
  });
  return response.ok;
}

export async function share(projectId: string): Promise<string | null> {
  const response = await fetch(`${API_ROOT}/projects/share/${projectId}`);
  if (!response.ok) return null;
  return await response.text();
}

export async function acceptShare(token: string): Promise<boolean> {
  const response = await fetch(`${API_ROOT}/projects/join/${token}`);
  return response.ok;
}

export async function updateMetadata(projectId: string, metadata: ProjectMetadata) {
  const response = await fetch(`${API_ROOT}/project/${projectId}/meta`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(metadata),
  });
  return response.ok
}

export type LoginRequest = {
  email: string,
  password: string,
}

export type CreateAccountRequest = {
  username: string,
  email: string,
  password: string,
  registration_token: string,
}

export type PrivateAccountInfo = {
  id: number,
  username: string,
  email: string,
  is_admin: boolean,
}

export type ProjectSummary = {
  id: string,
  title: string,
  owner: User,
  members: User[],
  created_at: string, // ISO Date
  updated_at: string, // ISO Date
  edited_by: User,
  metadata: ProjectMetadata | null,
}

export type ProjectMetadata = {
  tags: string[],
}

export function formatTimeAgo(dateObj: any): string {
  if (!dateObj) return 'Missing Data';

  try {
    const arr = Array.isArray(dateObj) ? dateObj : Object.values(dateObj);
    if (!arr || arr.length < 5) {
      return 'Invalid Format';
    }

    const year = Number(arr[0]);
    const dayOfYear = Number(arr[1]);
    const hour = Number(arr[2]);
    const minute = Number(arr[3]);
    const second = Number(arr[4]);
    const date = new Date(Date.UTC(year, 0, 1, hour, minute, second));
    date.setUTCDate(date.getUTCDate() + (dayOfYear - 1));

    if (Number.isNaN(date.getTime())){
      return 'Invalid Date';
    }

    const seconds = Math.max(0, Math.floor((Date.now() - date.getTime()) / 1000));
    if (seconds < 60) {
      return 'just now';
    }

    const timeUnits: [string, number][] = [
      ['year', 365 * 24 * 60 * 60],
      ['month', (365 / 12) * 24 * 60 * 60],
      ['week', 7 * 24 * 60 * 60],
      ['day', 24 * 60 * 60],
      ['hour', 60 * 60],
      ['minute', 60],
    ];

    for (const [unit, unitSeconds] of timeUnits) {
      const count = Math.floor(seconds / unitSeconds);
      if (count >= 1) {
        return `${count} ${unit}${count > 1 ? 's' : ''} ago`;
      }
    }

    return date.toLocaleDateString();
  } catch {
    return 'Error';
  }
}

export type User = {
  id: number,
  name: string,
}

export type ProjectOverviewIndex = {
  projects: ProjectSummary[],
}

export interface UserInvite {
  registration_token: string,
  expires: number, // TODO: check actual type
}
