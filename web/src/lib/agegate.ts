// The 18+ acknowledgement (docs/100_IDEAS.md #08).
//
// A pure, testable core: what counts as already-acknowledged, and what gets
// stored when a visitor accepts. The overlay component is a thin shell over
// these three functions.
//
// This is an ACKNOWLEDGEMENT, not an access control. Nothing server-side
// depends on it and the terms page says so plainly: a URL bar defeats any
// client-side wall, an interstitial that pretends otherwise is theatre, and
// claiming enforceability the site does not have would be worse than no gate.

export const AGE_KEY = 'sf_age_ok';

/** A visitor who has not acknowledged, or whose stored value is anything but
 *  the exact accepted marker, sees the overlay. Anything unrecognised is
 *  treated as "not acknowledged" rather than as consent. */
export function shouldGate(stored: string | null | undefined): boolean {
	return stored !== '1';
}

/** The value written on accept — localStorage (per browser) and a cookie
 *  (survives a fresh tab when storage is partitioned). */
export function acceptedValue(): string {
	return '1';
}

/** Cookie attributes for the acknowledgement. `SameSite=Lax` and a year, with
 *  no expiry-based re-ask: the content does not change with time. */
export function cookieString(maxAgeDays = 365): string {
	return `${AGE_KEY}=${acceptedValue()}; Path=/; Max-Age=${maxAgeDays * 24 * 60 * 60}; SameSite=Lax`;
}
