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

export async function invite(): Promise<UserInvite | null> {
  const response = await fetch(`${API_ROOT}/invite`);
  if (response.ok) return await response.json();
  return null;
}


export async function listProjects(): Promise<ProjectOverviewIndex | null> {
  const response = await fetch(`${API_ROOT}/projects`);
  if (response.ok) return await response.json();
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
}

export type ProjectSummary = {
  id: string,
  title: string,
  owner: User,
}

export type User = {
  id: number,
  name: string,
}

export type ProjectOverviewIndex = {
  projects: ProjectSummary[];
}

export interface UserInvite {
  registration_token: string,
  expires: number, // TODO: check actual type
}
