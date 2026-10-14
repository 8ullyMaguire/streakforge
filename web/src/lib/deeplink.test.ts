// Deep-link parsing (item 14). The two rules that matter beyond formatting are
// the fallback for an unknown kind and the open-redirect guard on `next=`.
import { describe, it, expect } from 'vitest';
import {
	parseKind,
	resolveKind,
	kindLabel,
	clampNote,
	safeNext,
	NOTE_MAX,
	LOG_KINDS
} from './deeplink';

describe('parseKind', () => {
	it('accepts every kind the API takes', () => {
		for (const k of LOG_KINDS) {
			expect(parseKind(k)).toEqual({ kind: k, fellBack: false });
		}
	});

	it('tolerates case and surrounding space', () => {
		expect(parseKind(' HABIT ')).toEqual({ kind: 'habit', fellBack: false });
		expect(parseKind('Denial')).toEqual({ kind: 'denial', fellBack: false });
	});

	it('falls back to a waste and REPORTS the fallback', () => {
		// Reporting it is the point: a page that silently recorded a waste
		// because a link said kind=waste would put something in a user's history
		// they never asked for.
		expect(parseKind('waste')).toEqual({ kind: 'habit', fellBack: true });
		expect(parseKind('')).toEqual({ kind: 'habit', fellBack: false });
		expect(parseKind(null)).toEqual({ kind: 'habit', fellBack: false });
		expect(parseKind(undefined)).toEqual({ kind: 'habit', fellBack: false });
	});
});

describe('resolveKind (the /log/<kind> path form)', () => {
	it('uses the path kind when there is no query', () => {
		// docs/100_IDEAS.md #14 names /log/habit literally, and that is the URL a
		// person types into a phone shortcut.
		expect(resolveKind('denial', null)).toEqual({ kind: 'denial', fellBack: false });
		expect(resolveKind('habit', '')).toEqual({ kind: 'habit', fellBack: false });
	});

	it('is identical to the query form when only the query is present', () => {
		// The path form must not change the behaviour of the URL that already worked.
		expect(resolveKind(null, 'denial')).toEqual(parseKind('denial'));
		expect(resolveKind(null, 'waste')).toEqual(parseKind('waste'));
	});

	it('REPORTS a link that contradicts itself instead of choosing silently', () => {
		// Path wins, but a page that quietly picked one of two answers would file
		// the other kind in someone's history without saying so.
		expect(resolveKind('denial', 'habit')).toEqual({
			kind: 'denial',
			fellBack: true,
			reason: 'mismatch'
		});
	});

	it('reports an unknown query kind beside a path kind', () => {
		expect(resolveKind('habit', 'banana')).toEqual({
			kind: 'habit',
			fellBack: true,
			reason: 'mismatch'
		});
	});

	it('agrees with itself without a warning when both name the same kind', () => {
		expect(resolveKind('affirmation', 'AFFIRMATION ')).toEqual({
			kind: 'affirmation',
			fellBack: false
		});
	});
});

describe('kindLabel', () => {
	it('names each kind in the site\'s own words', () => {
		expect(kindLabel('habit')).toBe('a waste');
		expect(kindLabel('denial')).toBe('a denial');
		expect(kindLabel('affirmation')).toBe('a drill rep');
	});
});

describe('clampNote', () => {
	it('passes a short note through, collapsed to one line', () => {
		expect(clampNote('  held   it  all day ')).toBe('held it all day');
	});

	it('truncates to the server limit', () => {
		const long = 'x'.repeat(NOTE_MAX + 40);
		expect(clampNote(long).length).toBe(NOTE_MAX);
	});

	it('counts code points, not UTF-16 units', () => {
		// An emoji outside the BMP is two UTF-16 units. Slicing by .length would
		// cut it in half and the server would reject the note as malformed.
		const emoji = '💦'.repeat(NOTE_MAX + 10);
		const out = clampNote(emoji);
		expect(Array.from(out).length).toBe(NOTE_MAX);
		expect(out).not.toContain('\uFFFD');
	});
});

describe('safeNext', () => {
	it('honours a same-site path', () => {
		expect(safeNext('/log?kind=denial')).toBe('/log?kind=denial');
	});

	it('refuses anything that could leave the site', () => {
		// A crafted /login?next=https://elsewhere.example must not turn the
		// sign-in page into an open redirect.
		expect(safeNext('https://elsewhere.example')).toBe('/dashboard');
		expect(safeNext('//elsewhere.example')).toBe('/dashboard');
		expect(safeNext('javascript:alert(1)')).toBe('/dashboard');
		expect(safeNext('')).toBe('/dashboard');
		expect(safeNext(null)).toBe('/dashboard');
	});
});
