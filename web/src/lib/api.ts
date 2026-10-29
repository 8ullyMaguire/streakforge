// API client — talks to the Rust backend at /api/*
import type {
	ApiError,
	AuthPayload,
	BoardsResponse,
	CageCheck,
	CageResponse,
	DenialResponse,
	DrillResponse,
	FeedItem,
	GuestStatus,
	KpiResponse,
	Leaderboard,
	LeaderboardPeriod,
	LockInfo,
	ModerationQueueItem,
	PublicFeedItem,
	LogKind,
	ManifestoList,
	Profile,
	SessionUser,
	Stats,
	TimeToCumResponse,
	Trust,
	UserOfTheDay
} from './types';
import { noteConsentRequiredFromApi } from './consent';

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
			const body = (await res.json()) as ApiError & { consent?: string; version?: number };
			// GS-3: the gate answers `403 {"consent":"required"}` with no prose,
			// because the caller is meant to act on it rather than print it. One
			// place turns that shape into a question the UI can show, so no
			// component has to recognise it.
			if (body.consent === 'required') {
				noteConsentRequiredFromApi();
				msg = 'Please choose your cookie settings first.';
			} else if (body.error) msg = body.error;
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
	nonce: () => request<{ nonce: string }>('/auth/nonce'),
	// GS-1: reading the door's state is free and does not mint anything.
	guestStatus: () => request<GuestStatus>('/guest'),
	mintGuest: () => request<SessionUser>('/guest', { method: 'POST' }),
	register: (payload: AuthPayload) => request<SessionUser>('/auth/register', { method: 'POST', body: JSON.stringify(payload) }),
	login: (payload: AuthPayload) => request<SessionUser>('/auth/login', { method: 'POST', body: JSON.stringify(payload) }),
	logHabit: (note?: string, kind: LogKind = 'habit', timeToCumSecs?: number) =>
		request<{ stats: Stats }>('/logs', {
			method: 'POST',
			body: JSON.stringify({
				note: note ?? '',
				kind,
				...(timeToCumSecs === undefined ? {} : { time_to_cum_secs: timeToCumSecs })
			})
		}),
	/** The one-click flow's second half: add or clear the duration, inside 15 min. */
	patchLog: (id: number, timeToCumSecs: number | null) =>
		request<{ stats: Stats }>(`/logs/${id}`, {
			method: 'PATCH',
			body: JSON.stringify({ time_to_cum_secs: timeToCumSecs })
		}),
	/** Undo, inside the same 15 minutes. */
	deleteLog: (id: number) => request<void>(`/logs/${id}`, { method: 'DELETE' }),
	cage: () => request<CageResponse>('/cage'),
	cageCheck: (note?: string) =>
		request<CageCheck>('/cage-check', {
			method: 'POST',
			body: JSON.stringify({ note: note ?? '' })
		}),
	cageQueue: () => request<CageCheck[]>('/cage-check/queue'),
	attest: (id: number, verdict: 'confirm' | 'dispute') =>
		request<CageCheck>(`/cage-check/${id}/attest`, {
			method: 'POST',
			body: JSON.stringify({ verdict })
		}),
	timeToCum: () => request<TimeToCumResponse>('/metrics/time-to-cum'),
	boards: () => request<BoardsResponse>('/metrics/boards'),
	trust: () => request<Trust>('/trust'),
	flag: (target_kind: 'log' | 'cage_check', target_id: number, reason: string) =>
		request<{ flags: number; from_tl2: number; hidden: boolean }>('/flags', {
			method: 'POST',
			body: JSON.stringify({ target_kind, target_id, reason })
		}),
	moderationQueue: () => request<ModerationQueueItem[]>('/moderation/queue'),
	moderationAction: (
		target_kind: 'log' | 'cage_check',
		target_id: number,
		action: 'hide' | 'restore',
		note?: string
	) =>
		request<{ hidden: boolean; restore_votes: number }>('/moderation/action', {
			method: 'POST',
			body: JSON.stringify({ target_kind, target_id, action, note })
		}),
	stats: () => request<Stats>('/stats'),
	drill: () => request<DrillResponse>('/drill'),
	denial: () => request<DenialResponse>('/denial'),
	lock: () => request<LockInfo>('/lock', { method: 'POST' }),
	unlock: (reason?: string) =>
		request<LockInfo>('/unlock', {
			method: 'POST',
			body: JSON.stringify({ reason: reason ?? '' })
		}),
	lockState: () => request<LockInfo>('/lock'),
	leaderboard: (period: LeaderboardPeriod) => request<Leaderboard>(`/leaderboard/${period}`),
	userOfTheDay: () => request<UserOfTheDay>('/user-of-the-day'),
	kpi: () => request<KpiResponse>('/kpi'),
	// The PUBLIC feed: notes included where the author left `logs_public` on, so the
	// type is PublicFeedItem, not the authenticated `FeedItem` the stats endpoint
	// returns. They were quietly the same type until the note came back.
	feed: (cursor?: string) =>
		request<{ items: PublicFeedItem[]; next_cursor: string | null }>(
			`/feed${cursor ? `?cursor=${encodeURIComponent(cursor)}` : ''}`
		),
	profile: (username: string) => request<Profile>(`/profile/${encodeURIComponent(username)}`),
	updateProfile: (data: {
		username?: string;
		display_name?: string;
		avatar_url?: string;
		social_url?: string;
		logs_public?: boolean;
	}) =>
		request<Profile>('/profile', { method: 'PATCH', body: JSON.stringify(data) }),
	manifestoList: () => request<ManifestoList>('/doctrine'),
	manifestoDoc: async (id: string) => {
		const res = await request<{ id: string; content: string }>(
			`/doctrine/${encodeURIComponent(id)}`
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
	return n.toString().padStart(9, '0');
}
