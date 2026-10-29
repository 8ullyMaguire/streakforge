/**
 * Durations, in one place.
 *
 * The site prints two very different clocks and they must not share a formatter:
 * a countdown to midnight (`formatCountdown`, in countdown.ts) and a *reported*
 * duration — how long a session ran (the owner's "time to cum": start to release).
 * The second one is the thing people compare on /metrics, so it is rendered the way
 * a stopwatch reads, not the way a lock timer reads.
 */

/** 900 -> "15m 00s"; 3725 -> "1h 02m"; 45 -> "45s". */
export function formatDuration(secs: number): string {
	const s = Math.max(0, Math.floor(secs));
	if (s < 60) return `${s}s`;
	const m = Math.floor(s / 60);
	const rem = s % 60;
	if (m < 60) return `${m}m ${String(rem).padStart(2, '0')}s`;
	const h = Math.floor(m / 60);
	return `${h}h ${String(m % 60).padStart(2, '0')}m`;
}

/** The same number, compact enough for a table cell: "15m", "1.5h", "3d". */
export function formatDurationShort(secs: number): string {
	const s = Math.max(0, Math.floor(secs));
	if (s < 60) return `${s}s`;
	const m = s / 60;
	if (m < 60) return `${Math.round(m)}m`;
	const h = m / 60;
	if (h < 24) return `${h.toFixed(1).replace(/\.0$/, '')}h`;
	return `${Math.round(h / 24)}d`;
}

/**
 * The chips under a fresh log: five common answers and a custom one.
 *
 * They are OFFERED, never required — the landing page logs on one click, and a
 * duration nobody supplied is a fact about the site (few people measure), not a
 * missing value to invent.
 */
export const TTC_CHIPS: { label: string; secs: number }[] = [
	{ label: '5m', secs: 300 },
	{ label: '15m', secs: 900 },
	{ label: '30m', secs: 1800 },
	{ label: '1h', secs: 3600 },
	{ label: '2h', secs: 7200 }
];

/** Parse a "12:30" or "45" style entry into seconds; null when it is not one. */
export function parseDurationInput(raw: string): number | null {
	const text = raw.trim().toLowerCase();
	if (!text) return null;
	let secs: number | null = null;
	const mmss = text.match(/^(\d{1,3}):([0-5]?\d)$/);
	const withUnit = text.match(/^(\d{1,3})\s*(m|min|h|hr|s|sec)$/);
	const plain = text.match(/^\d{1,5}$/);
	if (mmss) {
		secs = Number(mmss[1]) * 60 + Number(mmss[2]);
	} else if (withUnit) {
		const n = Number(withUnit[1]);
		const unit = withUnit[2];
		secs = unit.startsWith('h') ? n * 3600 : unit.startsWith('s') ? n : n * 60;
	} else if (plain) {
		secs = Number(plain[0]) * 60; // a bare number means minutes
	}
	// The same ceiling the server enforces: a day. Anything above it is a typo,
	// and storing it would be inventing a number the author did not mean.
	if (secs === null || secs < 0 || secs > 86400) return null;
	return secs;
}
