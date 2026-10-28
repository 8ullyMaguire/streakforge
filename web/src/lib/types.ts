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
	/** never-decaying all-time log count; survives the 30-day score decay */
	lifetime_total: number;
	last_log_at: string | null;
}

/** One entry of the PUBLIC feed: deliberately no `note`.
 *
 *  The confession box is private and only the author ever reads a note back
 *  (via `Stats.recent_logs`, which is their own). The backend's `activity_feed()`
 *  does not return a note column at all, so this type cannot carry one. */
export interface PublicFeedItem {
	id: string;
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	logged_at: string;
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
	/** every log ever made; never decays. A record, not a rankable score. */
	lifetime_total: number;
	/** true when the user has logs but none that still count: 30+ days idle and
	 *  the score has been wiped. */
	decayed: boolean;
	/** Whole days of silence left before the score is wiped, i.e.
	 *  `30 - days since the newest log`. `null` for a user who has never logged
	 *  (nothing to lose); `0` once the era has already ended. */
	days_until_decay: number | null;
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
	/** submissions this kind allows per UTC day (the server's own budget:
	 *  WLW_DAILY_LIMIT for a waste, 1 for a denial, DAILY_LIMIT for reps) */
	daily_limit: number;
	/** submissions still available for this kind today (never negative) */
	remaining_today: number;
	/** seconds until the daily cadence resets (next UTC midnight); 0 when allowed now */
	next_allowed_in_secs: number;
	/** Seconds until the daily budget refills (next UTC midnight). Unlike
	 * `next_allowed_in_secs` this is never 0 just because a log is allowed now,
	 * which is what the dashboard's "resets in 4h 12m" needs. */
	secs_until_reset: number;
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
	/** GS-1: a guest identity has no credential and is foldable into an account. */
	is_guest?: boolean;
}

/** GS-1: whether the guest door is open, and how close it is to closing. */
export interface GuestStatus {
	enabled: boolean;
	is_guest: boolean;
	mau: number;
	limit: number;
	window_days: number;
	at_cap: boolean;
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

export interface KpiTotals {
	total_wasted: number;
	total_denied: number;
	total_affirmations: number;
	total_users: number;
	active_24h: number;
	new_7d: number;
	currently_locked: number;
	total_lock_hours: number;
	denial_rate: number;
}

export interface KpiTrendPoint {
	day: string;
	wasted: number;
	denied: number;
	affirmations: number;
}

export interface KpiResponse {
	totals: KpiTotals;
	trend: KpiTrendPoint[];
	/** Reuses the leaderboard entry shape verbatim: the endpoint returns the
	 *  same struct, so a second type here would be a second thing to drift. */
	top_weekly: LeaderboardEntry[];
}

/** GET /api/today — what the community has done today (UTC).
 *
 *  Public, and already derivable from the KPI page; the point of the line on the
 *  landing page is that a solitary practice has a crowd visible next to it. */
export interface TodayCounts {
	denials_today: number;
	wastes_today: number;
	loggers_today: number;
}

export interface ApiError {
	error: string;
}

export function isApiError(x: unknown): x is ApiError {
	return typeof x === 'object' && x !== null && 'error' in x;
}
