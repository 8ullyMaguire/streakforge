// The ladder's words. What is tested here is the conversion, and the thing that broke
// before it existed: the moderation page printed "7 active days" for level 2 while the
// database granted it at 3, because the requirement was hardcoded in the template.

import { describe, expect, it } from 'vitest';
import { describeRung, ageLabel, bandNote } from './trust';
import type { TrustRung } from './types';

const rung = (over: Partial<TrustRung> = {}): TrustRung => ({
	level: 2,
	min_age_hours: 48,
	min_active_days: 3,
	min_contributions: 5,
	...over
});

describe('ageLabel', () => {
	it('says a day, not 24h', () => {
		expect(ageLabel(24)).toBe('a day');
		expect(ageLabel(48)).toBe('2 days');
	});
	it('reaches for the largest unit that fits exactly', () => {
		expect(ageLabel(168)).toBe('a week');
		expect(ageLabel(720)).toBe('a month');
		expect(ageLabel(1440)).toBe('2 months');
	});
	it('keeps hours when they are not a whole day', () => {
		expect(ageLabel(36)).toBe('36h');
	});
	it('handles the degenerate cases rather than printing NaN', () => {
		expect(ageLabel(0)).toBe('no wait');
		expect(ageLabel(-1)).toBe('—');
		expect(ageLabel(Number.NaN)).toBe('—');
	});
});

describe('describeRung', () => {
	it('describes level 1 by age and logs', () => {
		expect(describeRung(rung({ level: 1, min_age_hours: 24, min_contributions: 1 }))).toBe(
			'a day old, 1 log'
		);
	});
	it('describes the middle rungs by span and volume', () => {
		expect(describeRung(rung())).toBe('3 active days, 5 contributions');
		expect(describeRung(rung({ level: 3, min_active_days: 8, min_contributions: 15 }))).toBe(
			'8 active days, 15 contributions'
		);
	});
	it('pluralises one without an s', () => {
		expect(describeRung(rung({ min_active_days: 1, min_contributions: 1 }))).toBe(
			'1 active day, 1 contribution'
		);
	});
	it('names the rungs that are not earned by numbers', () => {
		expect(describeRung(rung({ level: 0 }))).toBe('any account');
		expect(describeRung(rung({ level: 4 }))).toBe('granted by hand');
	});
});

describe('bandNote', () => {
	it('explains the soft band and says it is temporary', () => {
		const note = bandNote('small', 1);
		expect(note).toContain('1 active member.');
		expect(note).toContain('tightens back by itself');
	});
	it('says nothing on the grown band', () => {
		expect(bandNote('grown', 25)).toBeNull();
	});
});
