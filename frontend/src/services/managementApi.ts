const _API_ROOT = '/api/v1'

export async function login(request: LoginRequest) {
    const response = await fetch(`${_API_ROOT}/login`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(request),
    });
    return response.ok
}

export async function register(request: CreateAccountRequest) {
    const response = await fetch(`${_API_ROOT}/register`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(request),
    });
    return response.ok
}

export async function whoami() {
    const response = await fetch(`${_API_ROOT}/login`);
    if (response.ok) {
        const json: PrivateAccountInfo = await response.json()
        return json;
    }
    return null;
}


export async function listProjects() {
    const response = await fetch(`${_API_ROOT}/projects`);
    const projects: ProjectIndex = await response.json();
    return projects;
}

export async function createProject(projectId: string) {
    const response = await fetch(`${_API_ROOT}/project/${projectId}`, {
        method: 'PUT',
    });
    return {
        success: response.ok,
        message: response.statusText,
    };
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
    projects: [ProjectSummary];
}