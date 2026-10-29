// The trust ladder's words, in one place.
//
// The numbers are never written here — `/api/trust` sends the rungs it actually judged the
// level against (0019: the ladder has a small-site band and a grown one). This module only
// turns those numbers into the sentence the moderation page prints, so "3 active days" can
// never become "7 active days" in the UI while the database still says three.
//
// Pure and dependency-free: the moderation page imports it, and so does its test.

import type { TrustRung } from './types';

/** `1 day`, `3 days`, `a week`, `2 months` — the reader should not have to divide 168. */
export function ageLabel(hours: number): string {
	if (!Number.isFinite(hours) || hours < 0) return '—';
	if (hours === 0) return 'no wait';
	if (hours % 720 === 0 && hours >= 720) {
		const m = hours / 720;
		return m === 1 ? 'a month' : `${m} months`;
	}
	if (hours % 168 === 0 && hours >= 168) {
		const w = hours / 168;
		return w === 1 ? 'a week' : `${w} weeks`;
	}
	if (hours % 24 === 0) {
		const d = hours / 24;
		return d === 1 ? 'a day' : `${d} days`;
	}
	return `${hours}h`;
}

/** What a rung asks for, as one clause: "2 days old, 5 contributions". */
export function describeRung(rung: TrustRung): string {
	if (rung.level <= 0) return 'any account';
	if (rung.level >= 4) return 'granted by hand';
	const contributions = `${rung.min_contributions} contribution${rung.min_contributions === 1 ? '' : 's'}`;
	// Level 1 is the only rung judged by account age; the rest are judged by activity.
	if (rung.level === 1) {
		const logs = `${rung.min_contributions} log${rung.min_contributions === 1 ? '' : 's'}`;
		return `${ageLabel(rung.min_age_hours)} old, ${logs}`;
	}
	// Both halves matter: active days describe the spread, contributions the volume. A
	// person who logs twelve times in one night has contributions and no tenure.
	const days = `${rung.min_active_days} active day${rung.min_active_days === 1 ? '' : 's'}`;
	return `${days}, ${contributions}`;
}

/** The sentence under the ladder when the site is small — the band is why the numbers are low. */
export function bandNote(band: string, activeAccounts: number): string | null {
	if (band !== 'small') return null;
	const who =
		activeAccounts === 1 ? '1 active member' : `${activeAccounts} active members`;
	return `SMALL SITE — ${who}. The ladder is softened (these are the numbers above) until ten people are logging; it tightens back by itself, with no deploy.`;
}
