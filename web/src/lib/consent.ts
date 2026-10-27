// GS-3 — the client half of the consent gate.
//
// The refusal that counts is the API's (`backend/src/consent.rs`): a mutation
// without consent is a 403 no matter what this module believes. This exists so a
// person is asked **once, in words**, and so a 403 arriving from a race — a
// session that expired in another tab, a cookie cleared by the browser — reopens
// the question instead of surfacing as a broken button.
//
// Where the refusal lives: `sessionStorage`, per tab, and nowhere else. The
// estate standard is explicit that a "no" must not be persisted like a setting
// the site chose for the visitor — so it is re-asked on the next visit, which is
// the honest behaviour for a site with an anonymous door.
//
// The pure parts are here rather than in the component so they can be tested
// without a DOM: which questions to ask, and what "this tab already refused"
// means.

import { writable } from 'svelte/store';

/** Optional categories, in the order the toast offers them. `essential` is not
 *  a choice — the session cookie is what keeps anyone signed in. */
export const OPTIONAL_CATEGORIES = ['analytics', 'personalisation'] as const;
export type OptionalCategory = (typeof OPTIONAL_CATEGORIES)[number];

export const REFUSED_KEY = 'sf_consent_refused';

export interface ConsentState {
	version: number;
	required: boolean;
	accepted: boolean;
	refused: boolean;
	categories: Record<string, boolean>;
}

/** Set when the API answers `403 {"consent":"required"}` — the toast subscribes. */
export const consentRequired = writable(false);

/** A refusal seen by the API is not this tab's refusal, but it is the same
 *  question: ask again rather than leaving the visitor with a dead button. */
export function noteConsentRequiredFromApi(): void {
	consentRequired.set(true);
}

/** The one question the toast asks itself. */
export function shouldAsk(state: ConsentState, tabRefused: boolean): boolean {
	return state.required && !state.accepted && !tabRefused;
}

type TabStorage = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

function tabStorage(): TabStorage | null {
	try {
		return typeof sessionStorage === 'undefined' ? null : sessionStorage;
	} catch {
		// A partitioned or blocked storage context throws on access rather than
		// returning null. Treating that as "not refused" asks the question again,
		// which is the safe direction: an unreadable "no" must never become a yes.
		return null;
	}
}

/** Has this tab already said no? Anything unreadable counts as "not yet". */
export function refusedThisTab(storage: TabStorage | null = tabStorage()): boolean {
	try {
		return storage?.getItem(REFUSED_KEY) === '1';
	} catch {
		return false;
	}
}

export function markRefusedThisTab(storage: TabStorage | null = tabStorage()): void {
	try {
		storage?.setItem(REFUSED_KEY, '1');
	} catch {
		// Nothing to do: the API still refuses, and the toast may ask again.
	}
}

export function clearRefusedThisTab(storage: TabStorage | null = tabStorage()): void {
	try {
		storage?.removeItem(REFUSED_KEY);
	} catch {
		// As above.
	}
}

const UNKNOWN: ConsentState = {
	version: 0,
	required: false,
	accepted: false,
	refused: false,
	categories: {}
};

/** What the server says. A network failure is reported as "nothing required":
 *  reads still work, and a mutation that needs consent will say so itself. */
export async function fetchConsentState(): Promise<ConsentState> {
	try {
		const res = await fetch('/api/consent', { credentials: 'include' });
		if (!res.ok) return UNKNOWN;
		return (await res.json()) as ConsentState;
	} catch {
		return UNKNOWN;
	}
}

/** Record the choice. Returns the server's answer, which is the record. */
export async function giveConsent(
	categories: readonly string[] = []
): Promise<ConsentState> {
	const res = await fetch('/api/consent', {
		method: 'POST',
		credentials: 'include',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ accept: true, categories })
	});
	if (!res.ok) throw new Error(`consent could not be recorded (${res.status})`);
	clearRefusedThisTab();
	consentRequired.set(false);
	return (await res.json()) as ConsentState;
}

/** Refuse. The site stays readable; this tab is not asked again. */
export async function refuseConsent(): Promise<ConsentState> {
	markRefusedThisTab();
	const res = await fetch('/api/consent', {
		method: 'POST',
		credentials: 'include',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ accept: false })
	});
	if (!res.ok) throw new Error(`consent could not be recorded (${res.status})`);
	consentRequired.set(false);
	return (await res.json()) as ConsentState;
}
