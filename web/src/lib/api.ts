// API client — talks to the Rust backend at /api/*
import type {
	ApiError,
	DrillResponse,
	FeedItem,
	Leaderboard,
	LeaderboardPeriod,
	LogKind,
	ManifestoList,
	Profile,
	SessionUser,
	Stats,
	UserOfTheDay
} from './types';

const BASE = '/api';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) },
		credentials: 'include',
		...init
	});
	if (!res.ok) {
		let msg = `Request failed (${res.status})`;
		try {
			const body = (await res.json()) as ApiError;
			if (body.error) msg = body.error;
		} catch {
			// non-JSON error body — keep default message
		}
		throw new ApiRequestError(msg, res.status);
	}
	if (res.status === 204) return undefined as T;
	return (await res.json()) as T;
}

export class ApiRequestError extends Error {
	status: number;
	constructor(message: string, status: number) {
		super(message);
		this.name = 'ApiRequestError';
		this.status = status;
	}
}

export const api = {
	me: () => request<SessionUser>('/auth/me'),
	logout: () => request<void>('/auth/logout', { method: 'POST' }),
	logHabit: (note?: string, kind: LogKind = 'habit') =>
		request<{ stats: Stats }>('/logs', {
			method: 'POST',
			body: JSON.stringify({ note: note ?? '', kind })
		}),
	stats: () => request<Stats>('/stats'),
	drill: () => request<DrillResponse>('/drill'),
	leaderboard: (period: LeaderboardPeriod) => request<Leaderboard>(`/leaderboard/${period}`),
	userOfTheDay: () => request<UserOfTheDay>('/user-of-the-day'),
	feed: (cursor?: string) =>
		request<{ items: FeedItem[]; next_cursor: string | null }>(
			`/feed${cursor ? `?cursor=${encodeURIComponent(cursor)}` : ''}`
		),
	profile: (username: string) => request<Profile>(`/profile/${encodeURIComponent(username)}`),
	updateProfile: (data: { username?: string; display_name?: string; avatar_url?: string }) =>
		request<Profile>('/profile', { method: 'PATCH', body: JSON.stringify(data) }),
	manifestoList: () => request<ManifestoList>('/manifesto'),
	manifestoDoc: async (id: string) => {
		const res = await request<{ id: string; content: string }>(
			`/manifesto/${encodeURIComponent(id)}`
		);
		return res.content;
	}
};

export function timeAgo(iso: string, now: Date = new Date()): string {
	const then = new Date(iso).getTime();
	const diff = Math.max(0, now.getTime() - then);
	const s = Math.floor(diff / 1000);
	if (s < 5) return 'just now';
	if (s < 60) return `${s}s ago`;
	const m = Math.floor(s / 60);
	if (m < 60) return `${m}m ago`;
	const h = Math.floor(m / 60);
	if (h < 24) return `${h}h ago`;
	const d = Math.floor(h / 24);
	if (d < 7) return `${d}d ago`;
	return new Date(iso).toLocaleDateString();
}

export function formatCount(n: number): string {
	return n.toString().padStart(8, '0');
}
