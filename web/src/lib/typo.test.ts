import { describe, it, expect } from 'vitest';
import {
	MAX_EDITS,
	normalizeAffirmation,
	levenshtein,
	matchesAffirmation,
	affirmationSimilarity
} from './typo';

describe('normalizeAffirmation', () => {
	it('lowercases and strips punctuation', () => {
		expect(normalizeAffirmation("Bnwo is not just a kink, it's reality!")).toBe(
			'bnwo is not just a kink it s reality'
		);
	});
	it('collapses whitespace', () => {
		expect(normalizeAffirmation('  I   submit!!  ')).toBe('i submit');
	});
	it('keeps digits', () => {
		expect(normalizeAffirmation('BNWO 2026')).toBe('bnwo 2026');
	});
});

describe('levenshtein', () => {
	it('handles identical, empty, and single edits', () => {
		expect(levenshtein('abc', 'abc')).toBe(0);
		expect(levenshtein('', 'abc')).toBe(3);
		expect(levenshtein('abc', 'ab')).toBe(1);
		expect(levenshtein('kitten', 'sitting')).toBe(3);
	});
});

describe('matchesAffirmation', () => {
	const target = 'Bnwo is not just a kink, it\u2019s reality, present and future';
	it('matches exact input', () => {
		expect(matchesAffirmation(target, target)).toBe(true);
	});
	it('matches case-insensitively', () => {
		expect(matchesAffirmation('BNWO IS NOT JUST A KINK, IT\u2019S REALITY, PRESENT AND FUTURE', target)).toBe(true);
	});
	it('matches when punctuation is missing', () => {
		expect(matchesAffirmation('Bnwo is not just a kink its reality present and future', target)).toBe(true);
	});
	it('accepts 1-2 typos', () => {
		expect(matchesAffirmation('Bnwo is not just a kink, its reality, present and futur', target)).toBe(true);
		expect(matchesAffirmation('Bnwo is not just a kink, its reality, presnt and future', target)).toBe(true);
	});
	it('rejects too many typos', () => {
		expect(matchesAffirmation('completely different text here', target)).toBe(false);
		expect(matchesAffirmation('', target)).toBe(false);
	});
	it('rejects empty target', () => {
		expect(matchesAffirmation('anything', '')).toBe(false);
	});
	it('MAX_EDITS is 3', () => {
		expect(MAX_EDITS).toBe(3);
	});
});

describe('affirmationSimilarity', () => {
	it('is 1 for exact normalized match', () => {
		expect(affirmationSimilarity('I submit!', 'i submit')).toBe(1);
	});
	it('is 0 for empty input', () => {
		expect(affirmationSimilarity('', 'i submit')).toBe(0);
	});
	it('is between 0 and 1 for partial', () => {
		const s = affirmationSimilarity('i submi', 'i submit');
		expect(s).toBeGreaterThan(0);
		expect(s).toBeLessThan(1);
	});
});
