export const API_ROOT = '/api/v1'

export async function login(request: LoginRequest) {
  const response = await fetch(`${API_ROOT}/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(request),
  });
  return response.ok
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

export async function whoami() {
  const response = await fetch(`${API_ROOT}/login`);
  if (response.ok) {
    const json: PrivateAccountInfo = await response.json()
    return json;
  }
  return null;
}

export async function invite(): Promise<UserInvite | null> {
  const response = await fetch(`${API_ROOT}/invite`);
  if (response.ok) return await response.json();
  return null;
}


export async function listProjects() {
  const response = await fetch(`${API_ROOT}/projects`);
  const projects: ProjectIndex = await response.json();
  return projects;
}

export async function createProject(projectId: string) {
  const response = await fetch(`${API_ROOT}/project/${projectId}`, {
    method: 'PUT',
  });
  return {
    success: response.ok,
    message: response.statusText,
  };
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

type LoginRequest = {
  email: string,
  password: string,
}

type CreateAccountRequest = {
  username: string,
  email: string,
  password: string,
  registration_token: string,
}

type PrivateAccountInfo = {
  username: string,
}

type ProjectSummary = {
  id: string,
}

type ProjectIndex = {
  projects: ProjectSummary[];
}

interface UserInvite {
  registration_token: string,
  expires: number, // TODO: check actual type
}
