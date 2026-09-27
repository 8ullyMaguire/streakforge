// Countdown formatting for the daily (UTC) cadence.
//
// The server sends `next_allowed_in_secs`, the number of seconds until the next
// UTC midnight at the moment the response was built. This module formats it and,
// crucially, computes it against a caller-supplied `now` so it can be TICKED
// down each second without a reload and without trusting the browser clock to
// agree with the server's.

// Returns the seconds remaining until the daily reset, given the server's
// original remaining count and how long we have been counting down.
// `serverSecs` is next_allowed_in_secs as sent; `elapsedMs` is time since the
// response arrived. Clamped at 0 so it never goes negative.
export function remainingSecs(serverSecs: number, elapsedMs: number): number {
	if (serverSecs <= 0) return 0;
	return Math.max(0, serverSecs - Math.floor(elapsedMs / 1000));
}

// HH:MM:SS, with hours allowed to exceed 24 so a long countdown still reads
// correctly. Used for the live ticking display.
export function formatCountdown(secs: number): string {
	const s = Math.max(0, Math.floor(secs));
	const h = Math.floor(s / 3600);
	const m = Math.floor((s % 3600) / 60);
	const sec = s % 60;
	return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
}

// Short human form for non-ticking contexts: "in 4h 12m", "in 9m", "now".
export function formatUntil(secs: number): string {
	if (secs <= 0) return 'now';
	const m = Math.ceil(secs / 60);
	if (m < 60) return `in ${m}m`;
	const h = Math.floor(m / 60);
	return `in ${h}h ${m % 60}m`;
}
