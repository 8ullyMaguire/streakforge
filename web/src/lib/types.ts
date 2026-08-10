// Shared types matching the Rust backend API (src/api.rs)
// Keep in sync with backend/src/api.rs response types.

export interface Profile {
	id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	social_url: string | null;
	created_at: string;
	streak: number;
	longest_streak: number;
	today_count: number;
	week_count: number;
	alltime_count: number;
}

export interface UserOfTheDay {
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	count: number;
	first_log_at: string;
	alltime_count: number;
}

export interface LeaderboardEntry {
	rank: number;
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	count: number;
	last_log_at: string | null;
}

export interface FeedItem {
	id: string;
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	logged_at: string;
	note: string | null;
}

export interface Stats {
	today_count: number;
	current_streak: number;
	longest_streak: number;
	week_count: number;
	alltime_count: number;
	recent_logs: FeedItem[];
	last_60m: number;
	can_log: boolean;
	next_allowed_at: string | null;
}

export interface DrillResponse {
	stats: Stats;
}

export type LogKind = 'habit' | 'affirmation';

export type LeaderboardPeriod = 'daily' | 'weekly' | 'alltime';

export interface Leaderboard {
	period: LeaderboardPeriod;
	entries: LeaderboardEntry[];
}

export interface SessionUser {
	id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	social_url?: string | null;
	provider?: string;
}

/** Payload sent to /api/auth/register and /api/auth/login. */
export interface AuthPayload {
	username: string;
	password: string;
	form_opened_at: number;
	website: string;
	challenge_proof: string;
	challenge_nonce: string;
}

export interface ManifestoDoc {
	id: string;
	title: string;
	filename: string;
}

export interface ManifestoList {
	docs: ManifestoDoc[];
}

export interface ApiError {
	error: string;
}

export function isApiError(x: unknown): x is ApiError {
	return typeof x === 'object' && x !== null && 'error' in x;
}
