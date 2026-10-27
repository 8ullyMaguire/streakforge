import { describe, expect, it } from 'vitest';
import {
	OPTIONAL_CATEGORIES,
	REFUSED_KEY,
	clearRefusedThisTab,
	markRefusedThisTab,
	refusedThisTab,
	shouldAsk,
	type ConsentState
} from './consent';

/** A sessionStorage stand-in. `sessionStorage` does not exist under vitest's
 *  node environment, which is also the reason the module takes it as a
 *  parameter instead of reading the global. */
function fakeStorage(initial: Record<string, string> = {}) {
	const map = new Map(Object.entries(initial));
	return {
		getItem: (k: string) => map.get(k) ?? null,
		setItem: (k: string, v: string) => void map.set(k, v),
		removeItem: (k: string) => void map.delete(k),
		dump: () => Object.fromEntries(map)
	};
}

function state(over: Partial<ConsentState> = {}): ConsentState {
	return {
		version: 1,
		required: true,
		accepted: false,
		refused: false,
		categories: { essential: true, analytics: false, personalisation: false },
		...over
	};
}

describe('shouldAsk', () => {
	it('asks a visitor who has neither accepted nor refused', () => {
		expect(shouldAsk(state(), false)).toBe(true);
	});

	it('does not ask after the API says consent was accepted', () => {
		expect(shouldAsk(state({ accepted: true }), false)).toBe(false);
	});

	it('does not re-ask a tab that refused, even though the server still refuses mutations', () => {
		// The server has no record of a refusal — that is the point — so the only
		// thing that stops a loop of "you must accept" is this per-tab flag.
		expect(shouldAsk(state(), true)).toBe(false);
	});

	it('does not ask at all when the gate is switched off', () => {
		expect(shouldAsk(state({ required: false }), false)).toBe(false);
	});

	it('asks again in a new tab after a refusal, because a refusal is not a setting', () => {
		// Same server state, fresh storage: refusedThisTab is per-tab by design.
		expect(shouldAsk(state(), refusedThisTab(fakeStorage()))).toBe(true);
	});
});

describe('the refusal flag', () => {
	it('is absent until a refusal is recorded', () => {
		expect(refusedThisTab(fakeStorage())).toBe(false);
	});

	it('round-trips a refusal and is cleared by accepting', () => {
		const storage = fakeStorage();
		markRefusedThisTab(storage);
		expect(storage.dump()[REFUSED_KEY]).toBe('1');
		expect(refusedThisTab(storage)).toBe(true);

		clearRefusedThisTab(storage);
		expect(refusedThisTab(storage)).toBe(false);
	});

	it('treats unreadable storage as "not refused" rather than as consent', () => {
		const throwing = {
			getItem: () => {
				throw new Error('blocked');
			},
			setItem: () => {
				throw new Error('blocked');
			},
			removeItem: () => {
				throw new Error('blocked');
			}
		};
		expect(refusedThisTab(throwing)).toBe(false);
		// And writing must not throw into the click handler.
		expect(() => markRefusedThisTab(throwing)).not.toThrow();
		expect(() => clearRefusedThisTab(throwing)).not.toThrow();
	});

	it('has no storage at all under a server render', () => {
		expect(refusedThisTab(null)).toBe(false);
		expect(() => markRefusedThisTab(null)).not.toThrow();
	});
});

describe('the optional categories', () => {
	it('are exactly the two the standard names, and essential is not among them', () => {
		expect(OPTIONAL_CATEGORIES).toEqual(['analytics', 'personalisation']);
	});
});
