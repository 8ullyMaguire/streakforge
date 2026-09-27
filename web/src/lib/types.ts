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
	points: number;
	first_log_at: string;
	alltime_count: number;
}

export interface LeaderboardEntry {
	rank: number;
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	/** Whiteboi Devotion Index: ((WLW + WLD + Game Bonus) x multiplier) - penalty */
	score: number;
	/** components, so a score can be explained on the board */
	wlw: number;
	wld: number;
	game_bonus: number;
	multiplier: number;
	racism_penalty: number;
	/** active exclusive streak (the most recent WLW or WLD) and which kind it is */
	streak: number;
	active_kind: string;
	/** legacy mirror of `wld`; the 💧 badge on both board pages reads it.
	 *  `points` / `waste_count` / `affirmation_count` went away with the 10x
	 *  weighting they encoded. */
	denial_count: number;
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

export interface DevotionIndex {
	wlw: number;
	wld: number;
	game_bonus: number;
	multiplier: number;
	/** always 0 today: no reporting data source exists for it */
	racism_penalty: number;
	score: number;
	/** 'habit' | 'denial' | 'none' — the kind holding the active exclusive streak */
	active_kind: string;
	active_streak: number;
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
	/** seconds until the daily cadence resets (next UTC midnight); 0 when allowed now */
	next_allowed_in_secs: number;
	/** true when the OTHER exclusive kind already ran today */
	blocked_by_exclusivity: boolean;
	devotion: DevotionIndex;
}

export interface DrillResponse {
	stats: Stats;
}

export interface LockInfo {
	locked: boolean;
	locked_at?: string | null;
	current_duration_secs: number;
	total_locked_secs: number;
	longest_lock_secs: number;
	current_lock_streak: number;
	longest_lock_streak: number;
}

export interface DenialResponse {
	stats: Stats;
	lock: LockInfo;
	/** seconds until the next denial can be reported, or null if allowed now */
	next_denial_allowed_in: number | null;
}

export type LogKind = 'habit' | 'affirmation' | 'denial';

/** MONTHLY (rolling 30 days) is the default board view. */
export type LeaderboardPeriod = 'daily' | 'weekly' | 'monthly' | 'alltime';

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
