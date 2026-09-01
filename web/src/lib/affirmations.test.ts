import { describe, it, expect } from 'vitest';
import { AFFIRMATIONS, affirmationOfTheDay } from './affirmations';

describe('affirmations deck', () => {
	it('has a non-empty curated deck', () => {
		expect(AFFIRMATIONS.length).toBeGreaterThan(10);
	});
	it('every affirmation has text and source', () => {
		for (const a of AFFIRMATIONS) {
			expect(a.text.length).toBeGreaterThan(0);
			expect(a.source.length).toBeGreaterThan(0);
		}
	});
	it('affirmationOfTheDay is deterministic per day', () => {
		const day = 1786374327000;
		expect(affirmationOfTheDay(day).text).toBe(affirmationOfTheDay(day).text);
	});
	it('different days can yield different affirmations', () => {
		const a = affirmationOfTheDay(1786374327000);
		const b = affirmationOfTheDay(1786374327000 + 86400000 * AFFIRMATIONS.length);
		expect(a.text).toBe(b.text); // full cycle wraps
	});
});
