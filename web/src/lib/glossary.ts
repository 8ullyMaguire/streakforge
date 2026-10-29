// The site's vocabulary, in one place.
//
// Data rather than markup so the glossary can be TESTED as content: the test
// checks that every term is defined, that no term appears twice, and that each
// entry names where the rule is enforced. A list of words on a page is easy to
// let rot; the same list as an exported array cannot quietly lose an entry
// without a test going red.
//
// The point of the page (docs/100_IDEAS.md #12) is that this site's nouns are
// load-bearing — "WLW" is a score, a budget, a streak and a board column — and a
// newcomer meets fifteen of them in the first minute.

export interface GlossaryTerm {
	/** The word as it appears in the UI. */
	term: string;
	/** What it means, in one or two sentences, in the site's own terms. */
	definition: string;
	/** Where it is enforced, so a reader knows the rule is real and not copy. */
	enforced: string;
}

export const TERMS: GlossaryTerm[] = [
	{
		term: 'WLW',
		definition:
			'A whiteboi load wasted. The log that spends your daily waste budget, and the only kind that appears in the public activity feed.',
		enforced: 'kind = \'habit\' in habit_logs; activity_feed() filters on it'
	},
	{
		term: 'WLD',
		definition:
			'A whiteboi load denied. Logged at most once per UTC day, and the only kind that opens a lock.',
		enforced: 'kind = \'denial\'; one per UTC day'
	},
	{
		term: 'Exclusivity',
		definition:
			'One-way, and the direction is the rule: a waste spends the day\'s denial, because a whiteboi who cums cannot claim he was denied. A denial spends nothing — the load you waste afterwards is exactly what the site is here to count.',
		enforced: 'the denial endpoint refuses after a waste; the waste endpoint never refuses after a denial'
	},
	{
		term: 'Devotion Index',
		definition:
			'(days of WLW + days of WLD + game bonus) × multiplier. Counts DAYS, not logs: five wastes in one day are one day of devotion. The score on every board.',
		enforced: 'devotion_index() — count(distinct log_date)'
	},
	{
		term: 'Multiplier',
		definition:
			'Earned by consecutive days of showing up — any kind, drills included: 1–6 days ×1.0, 7–13 ×1.25, 14–29 ×1.5, 30–59 ×2.0, 60–99 ×2.5, 100+ ×3.0. Changing what you do each day never costs it; missing a day does. This is why consistency beats volume.',
		enforced: 'devotion_multiplier(user_participation_streak(p_user))'
	},
	{
		term: 'Active streak',
		definition:
			'Consecutive UTC days of the kind you logged most recently — a waste or a denial. Drill reps never hold one. This streak resets when you switch kind; the multiplier does not.',
		enforced: 'exclusive_streak(p_user)'
	},
	{
		term: 'Game bonus',
		definition:
			'One point per three drill reps (floor of affirmations ÷ 3). Repetition is rewarded as an exercise, never as a score.',
		enforced: 'devotion_index(): floor(count(affirmations) / 3)'
	},
	{
		term: 'Decay',
		definition:
			'Thirty days without any log wipes the score to zero. The dashboard warns you in the last ten days rather than telling you afterwards.',
		enforced: 'devotion_index(): the 30-day horizon'
	},
	{
		term: 'Lifetime total',
		definition:
			'Every log you have ever made. It never decays and nothing sorts by it: it is a record, not a rank.',
		enforced: 'devotion_lifetime(p_user)'
	},
	{
		term: 'Waste budget',
		definition:
			'Five wastes per UTC day by default on this deployment. It is a budget, not a ration: unused wastes do not carry over and are not a failure.',
		enforced: 'WLW_DAILY_LIMIT (5 here)'
	},
	{
		term: 'Denial cadence',
		definition:
			'One denial per UTC day, and it does not spend the waste budget. A waste recorded first does spend the denial\'s day, so report the stand before you fall.',
		enforced: 'the denial endpoint, per UTC day'
	},
	{
		term: 'Drill',
		definition:
			'The affirmation reps. An exercise, with its own daily allowance, that feeds the game bonus and never the exclusive streak — but a drill day still counts as a day of showing up for the multiplier.',
		enforced: 'kind = \'affirmation\'; DAILY_LIMIT'
	},
	{
		term: 'Lock',
		definition:
			'A timed denial session you open and record. It is not scored — it is kept, and the total time is.',
		enforced: 'lock_sessions; kpi_totals().total_lock_hours'
	},
	{
		term: 'Streak',
		definition:
			'Consecutive UTC days with at least one log of any kind. The daily cadence starts at 00:00 UTC, so a log at 23:59 belongs to today.',
		enforced: 'user_streak(p_user)'
	},
	{
		term: 'Board age floor',
		definition:
			'An account must be a day old before it can hold a place on any board. Making a second account to top one is not free.',
		enforced: 'devotion_leaderboard(): created_at <= now() - 24h'
	},
	{
		term: 'Note',
		definition:
			'A private 140-character confession attached to a log. Only you can read it back, on your own dashboard.',
		enforced: 'the public feed does not return a note column at all'
	},
	{
		term: 'Doctrine',
		definition: 'The collected texts. Reading them is free and counts for nothing, which is the point.',
		enforced: 'manifestos/*.md'
	}
];
