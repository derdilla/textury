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

type LoginRequest = {
    email: String,
    password: String,
}

type CreateAccountRequest = {
    username: String,
    email: String,
    password: String,
    registration_token: String,
}

type PrivateAccountInfo = {
    username: String,
}