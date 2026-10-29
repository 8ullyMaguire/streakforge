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
	/** Whether this member's confessions appear in the public feed. True by
	 *  default; the switch lives in settings. */
	logs_public: boolean;
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

/** One entry of the PUBLIC feed.
 *
 *  Since 2026-10-29 a confession is PUBLIC by default and its author can turn that
 *  off in settings. `note` is therefore present-or-absent depending on that switch,
 *  and `activity_feed()` — not this type — is what enforces it. `kind` says which
 *  event it was: a waste or a denial (drills never appear). */
export interface PublicFeedItem {
	id: string;
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	kind: 'habit' | 'denial';
	note: string | null;
	logged_at: string;
	/** Filed late for the day it describes (a catch-up). */
	catch_up: boolean;
}

export interface FeedItem {
	id: string;
	user_id: string;
	username: string;
	display_name: string | null;
	avatar_url: string | null;
	logged_at: string;
	note: string | null;
	/** Filed late for the day it describes (the yesterday window). */
	catch_up: boolean;
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
	/** The caller's own time-to-cum figures — the owner's definition: how long the
	 *  session ran, from starting to release. */
	time_to_cum: TimeToCumMine;
	/** Each kind's own exclusive streak, side by side. */
	kind_streaks: { habit: number; denial: number; affirmation: number };
	/** The multiplier tiers with which ones this run already reached. */
	milestones: MilestoneTier[];
	/** True when skipping today costs the multiplier already earned. */
	at_risk: AtRisk;
	/** Whether yesterday is empty and a catch-up has not been filed today. The
	 *  server decides; the page prints what it is told. */
	catch_up: CatchUpState;
	devotion: DevotionIndex;
}

export interface TimeToCumMine {
	samples: number;
	avg_secs: number;
	median_secs: number;
	min_secs: number;
	max_secs: number;
	last_secs: number | null;
}

export interface MilestoneTier {
	days: number;
	multiplier: number;
	reached: boolean;
}

export interface CatchUpState {
	available: boolean;
	filed_today: boolean;
	yesterday_count: number;
}

export interface AtRisk {
	at_risk: boolean;
	secs_left: number;
	multiplier: number;
	streak: number;
	next_tier_in: number | null;
}

export interface CageMetrics {
	checks: number;
	claimed_secs: number;
	verified_secs: number;
	pending_secs: number;
	disputed_secs: number;
	verification_rate: number;
	avg_secs: number;
	current_verified_day_streak: number;
	longest_verified_day_streak: number;
	last_check_at: string | null;
}

export interface CageCheck {
	id: number;
	username: string;
	avatar_url: string | null;
	checked_at: string;
	seal_code: string;
	note: string | null;
	claimed_secs: number;
	status: 'pending' | 'verified' | 'disputed';
	confirms: number;
	disputes: number;
	my_verdict: 'confirm' | 'dispute' | null;
}

export interface CageResponse {
	lock: LockInfo;
	metrics: CageMetrics;
	today: CageCheck | null;
	recent: CageCheck[];
	queue_count: number;
	total_verified_secs: number;
}

export interface TimeToCumAggregate {
	samples: number;
	avg_secs: number;
	median_secs: number;
	avg_of_user_avgs_secs: number;
	reporters: number;
}

export interface EdgeEntry {
	username: string;
	avatar_url: string | null;
	samples: number;
	avg_secs: number;
}

export interface TimeToCumResponse {
	aggregate: TimeToCumAggregate;
	mine: TimeToCumMine | null;
	board: EdgeEntry[];
}

export interface BoardsResponse {
	most_denied: { username: string; avatar_url: string | null; denials: number }[];
	consistency: { username: string; avatar_url: string | null; days: number; pct: number }[];
	locks: {
		username: string;
		avatar_url: string | null;
		current_streak: number;
		longest_streak: number;
	}[];
}

/** One rung of the ladder as the server publishes it. The numbers are never hardcoded
 * here: `/api/trust` sends the thresholds the level was actually judged against, so the
 * moderation page cannot print a requirement the function does not enforce. */
export interface TrustRung {
	level: number;
	min_age_hours: number;
	min_active_days: number;
	min_contributions: number;
}

export interface Trust {
	level: number;
	active_days: number;
	contributions: number;
	upheld_flags: number;
	next_level_at: number | null;
	is_leader: boolean;
	/** `small` below ten active members, `grown` at or above it. */
	band: 'small' | 'grown' | 'unknown';
	active_accounts: number;
	thresholds: TrustRung[];
}

export interface DeckItem {
	id: number;
	body: string;
	source: string;
	author: string;
	mine: boolean;
	status: 'live' | 'pending' | 'hidden';
	created_at: string;
}

export interface DeckResponse {
	deck: DeckItem[];
	mine: DeckItem[];
	review: DeckItem[];
	can_review: boolean;
	level: number;
	review_level: number;
	remaining_today: number | null;
	max_len: number;
}

export interface SubmitAffirmationResponse {
	id: number;
	body: string;
	status: 'live' | 'pending';
	live: boolean;
	remaining_today: number;
}

export interface ModerationQueueItem {
	target_kind: 'log' | 'cage_check';
	target_id: number;
	flags: number;
	from_tl2: number;
	hidden: boolean;
	preview: string | null;
	author: string | null;
	first_flag_at: string;
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
