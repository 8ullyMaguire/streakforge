import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { api, ApiRequestError, timeAgo, formatCount } from './api';

// Mock fetch so api.* tests don't need a backend.
function mockFetch(handler: (url: string, init?: RequestInit) => { status: number; body: unknown }) {
	return vi.fn(async (url: string, init?: RequestInit) => {
		const { status, body } = handler(url, init);
		return {
			ok: status >= 200 && status < 300,
			status,
			json: async () => body
		} as Response;
	});
}

beforeEach(() => {
	vi.stubGlobal('fetch', mockFetch(() => ({ status: 200, body: {} })));
});

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('api auth endpoints', () => {
	it('nonce() GETs /api/auth/nonce and returns the nonce', async () => {
		vi.stubGlobal('fetch', mockFetch(() => ({ status: 200, body: { nonce: 'abc123' } })));
		const res = await api.nonce();
		expect(res).toEqual({ nonce: 'abc123' });
		expect(fetch).toHaveBeenCalledWith('/api/auth/nonce', expect.objectContaining({ credentials: 'include' }));
	});

	it('register() POSTs the full auth payload to /api/auth/register', async () => {
		vi.stubGlobal('fetch', mockFetch(() => ({ status: 200, body: { id: 'u1', username: 'alice' } })));
		const payload = {
			username: 'alice',
			password: 'password123',
			form_opened_at: 1720000000000,
			website: '',
			challenge_proof: '6309588508e5f3b4',
			challenge_nonce: 'deadbeef'
		};
		const res = await api.register(payload);
		expect(res).toEqual({ id: 'u1', username: 'alice' });
		const [, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0] as [string, RequestInit];
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(payload);
	});

	it('login() POSTs the same payload shape to /api/auth/login', async () => {
		vi.stubGlobal('fetch', mockFetch(() => ({ status: 200, body: { id: 'u1', username: 'alice' } })));
		const payload = {
			username: 'alice',
			password: 'password123',
			form_opened_at: 1720000000000,
			website: '',
			challenge_proof: '6309588508e5f3b4',
			challenge_nonce: 'deadbeef'
		};
		await api.login(payload);
		const [, init] = (fetch as ReturnType<typeof vi.fn>).mock.calls[0] as [string, RequestInit];
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual(payload);
		expect((fetch as ReturnType<typeof vi.fn>).mock.calls[0][0]).toBe('/api/auth/login');
	});

	it('surfaces the API error message on 4xx', async () => {
		vi.stubGlobal('fetch', mockFetch(() => ({ status: 422, body: { error: 'Challenge proof invalid' } })));
		await expect(api.login({} as never)).rejects.toThrow(ApiRequestError);
		await expect(api.login({} as never)).rejects.toThrow('Challenge proof invalid');
	});
});

describe('timeAgo', () => {
	const now = new Date('2026-08-10T12:00:00Z');
	it('returns just now for < 5s', () => {
		expect(timeAgo('2026-08-10T11:59:58Z', now)).toBe('just now');
	});
	it('seconds', () => {
		expect(timeAgo('2026-08-10T11:59:30Z', now)).toBe('30s ago');
	});
	it('minutes', () => {
		expect(timeAgo('2026-08-10T11:45:00Z', now)).toBe('15m ago');
	});
	it('hours', () => {
		expect(timeAgo('2026-08-10T09:00:00Z', now)).toBe('3h ago');
	});
	it('days', () => {
		expect(timeAgo('2026-08-07T12:00:00Z', now)).toBe('3d ago');
	});
	it('older than a week uses locale date', () => {
		expect(timeAgo('2026-07-01T12:00:00Z', now)).toBe(new Date('2026-07-01T12:00:00Z').toLocaleDateString());
	});
});

describe('formatCount', () => {
	it('pads to 9 digits', () => {
		expect(formatCount(62)).toBe('000000062');
	});
	it('handles large numbers', () => {
		expect(formatCount(123456789)).toBe('123456789');
	});
});
