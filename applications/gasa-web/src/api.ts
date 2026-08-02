const BASE = window.ENV?.API_URL ?? ''

export class ApiError extends Error {
  status: number
  code: string

  constructor(status: number, code: string) {
    super(code)
    this.status = status
    this.code = code
  }
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(BASE + path, {
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json' },
    ...options,
  })

  if (res.status === 401) {
    // Session expired: restart the oauth2-proxy login flow and return here
    window.location.assign(
      '/oauth2/start?rd=' + encodeURIComponent(window.location.href),
    )
    throw new ApiError(401, 'unauthorized')
  }

  if (!res.ok) {
    let code = 'error'
    try {
      const body = await res.json()
      if (typeof body?.error === 'string') code = body.error
    } catch {
      // non-JSON error body
    }
    throw new ApiError(res.status, code)
  }

  if (res.status === 204) {
    return undefined as T
  }
  return res.json() as Promise<T>
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, body?: unknown) =>
    request<T>(path, {
      method: 'POST',
      body: body === undefined ? undefined : JSON.stringify(body),
    }),
  delete: <T>(path: string) => request<T>(path, { method: 'DELETE' }),
}
